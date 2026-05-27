# Feature Research: v4.0 Platform Maturity

**Domain:** Calculator emulator platform — skin theming, onboarding/reference, GUI keyboard parity, binary file exchange (.raw), Extended Memory (X-MEM)
**Researched:** 2026-05-27
**Confidence:** MEDIUM-HIGH — HP-41 .raw format details are MEDIUM confidence (HP Museum byte table is 403-blocked; specifications reconstructed from multiple secondary sources including the V41 help file, Synthetic QRG content, and HP-41 community documentation); X-MEM file type architecture is MEDIUM confidence (reconstructed from V41 Help.txt, Wikipedia, and HP Museum forum discussions); theming and onboarding patterns are HIGH confidence (well-documented web standards).

---

## Domain Overview

v4.0 adds five orthogonal capability areas to a feature-complete HP-41 emulator (v3.3 shipped 325 Op variants, 5 XROM modules). Each area is independently scoped:

1. **Skin themes** — CSS variable retheming of the Tauri v2/React GUI
2. **Onboarding/reference** — first-run guide + searchable in-app function reference beyond `?` overlay
3. **GUI keyboard parity** — close physical keyboard shortcut gaps that CLI covers but GUI does not
4. **`.raw` file I/O** — HP-41 community binary program exchange format
5. **Extended Memory (X-MEM)** — EMDIR/EMROOM/EMREG operations on a named-file register store

---

## Area 1: Skin Themes

### Background

The GUI currently hardcodes ~54 hex color literals across `App.css` (394 lines) and one color reference in `Keyboard.tsx`. The calculator body background, key fills, display background, and annotation colors are all baked in as dark-theme values (e.g. `#0d0d0d` body, `#c8e6c9` display text, orange/blue shift/alpha key label colors). There are no CSS custom properties (`--var`) in use today.

Free42 (the HP-42S emulator) implements external skin support via a `.layout` descriptor file pointing to a `.gif` bitmap. This is the gold standard in the HP calculator emulator community, but it is more complex than what v4.0 needs: Free42 skins remap clickable regions, add keyboard mappings, and support custom display colors per skin — all requiring a skin descriptor language. The HP-41 GUI SVG keyboard is code-generated in `Keyboard.tsx`, so bitmap skins don't apply.

The right model for this codebase is **CSS custom property theming**: define a palette of ~20–30 CSS variables on `:root`, override per theme via `data-theme="light"` / `data-theme="beige"` / `data-theme="hi-contrast"` on `<html>`, and persist the choice in `~/.hp41/settings.json` (or piggyback on `autosave.json`).

### Table Stakes

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| Dark theme (default) | Already ships; just needs CSS-variable extraction | LOW | Current hardcoded colors become the dark theme values |
| Light theme | Users in bright environments expect this | LOW | Invert display/background; keep key label colors |
| Classic beige / "HP-41" theme | Retro computing users expect hardware-authentic palette | MEDIUM | Cream key bodies (#d4c9a8), dark-charcoal legends (#2a2a2a), amber LCD |
| Theme persistence across restarts | Users don't want to re-select every session | LOW | One extra JSON field in `~/.hp41/settings.json` |

### Differentiators

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| High-contrast accessibility theme | WCAG AA compliance (4.5:1 contrast ratio) for accessibility users | LOW | White-on-black with high-saturation labels; no new infrastructure |
| Smooth theme transition | Avoids jarring flash when switching | LOW | `transition: background-color 150ms ease` on root |
| System theme auto-follow (prefers-color-scheme) | Zero-effort dark/light matching | LOW | CSS `@media (prefers-color-scheme: dark)` check + manual override |

### Anti-Features

| Anti-Feature | Why Requested | Why Problematic | Alternative |
|--------------|---------------|-----------------|-------------|
| External skin file loading (.gif bitmap) | Free42-style community skin library | SVG keyboard is code-generated; bitmap remapping requires hot-spot coordinate files and a parser. Complexity disproportionate to v4.0 scope | 3-4 built-in CSS themes cover 95% of user needs |
| Per-key color overrides | Maximum customization | Multiplies the surface area from ~20 palette variables to 150+ per-key rules | Palette variables cascade naturally to all keys |
| Theme editor UI | Power user appeal | Non-trivial UI surface; color pickers need Tauri window permissions; deferred | Ship 3-4 presets; community can submit PRs |

### CSS Variable Surface

Minimum viable palette (~22 variables):

```
--color-body-bg          Calculator body background
--color-border           Border around calculator
--color-display-bg       LCD background
--color-display-fg       LCD text / segment color
--color-annunciator-dim  Inactive annunciator dimmed color
--color-annunciator-on   Active annunciator bright color
--color-stack-bg         Stack panel background
--color-stack-text       Stack register values
--color-stack-label      X/Y/Z/T/L labels
--color-key-body         Main key fill
--color-key-text         Primary key label (white)
--color-key-shift        Shifted function label (orange on authentic)
--color-key-alpha        Alpha letter label (blue on authentic)
--color-key-pressed      Key pressed state fill
--color-key-top-body     Top-row mode key fill (USER/PRGM/ALPHA)
--color-key-top-text     Top-row mode key label
--color-print-bg         Print panel background
--color-print-text       Print panel text
--color-toast-bg         Toast error background
--color-toast-text       Toast error text
--color-modal-bg         Help/modal overlay background
--color-modal-text       Help/modal overlay text
```

### Dependencies

- No `hp41-core` changes required.
- No new Tauri commands required.
- Theme persistence requires either: a new `settings.json` file in `~/.hp41/` (simplest) or a new `theme: String` field in `CalcState` (not recommended — theme is UI state, not calculator state).
- The Keyboard.tsx SVG color scheme is currently set by key CSS class fills. The SHIFT key color and key body color are hardcoded inline styles in SVG `<rect>` elements — these must be converted to CSS-variable references on the `.key` class.

---

## Area 2: Onboarding / In-App Reference

### Background

The current `?` help overlay (CLI + GUI) is a searchable function catalog showing key positions, function names, and XROM module assignments. It is a reference tool, not a tutorial. New users unfamiliar with RPN face a steep entry curve: no mode indicators explain what USER/PRGM/ALPHA do, there is no introductory walkthrough, and the 325-Op function set overwhelms first-timers.

Comparative analysis:
- **Free42** has no onboarding; relies on the HP-42S Owner's Manual PDF.
- **V41** has no onboarding; ships a plain-text Help.txt.
- **HP-15C web emulator (hp15c.com)** surfaces a `h` key that shows keyboard labels — minimal.
- **cs-41 RPN Calculator (macOS)** has no onboarding.

None of the major HP-41 emulators have meaningful onboarding. This is a genuine differentiator.

### Table Stakes

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| First-run "Welcome" modal | New users are confused without RPN orientation | LOW | Detect first run via `~/.hp41/settings.json`; show once |
| RPN quick-start guide (3-5 steps) | RPN is unfamiliar to most modern users | LOW | Static modal with "Enter a number → Enter → another number → +" walkthrough |
| `?` overlay search improvement | Current search is basic substring match; users expect ranked/fuzzy results | MEDIUM | Existing `help_entries_all()` already has 350+ entries; improve ranking |
| Close help overlay with Escape | Basic UX expectation; currently works, should be verified on all platforms | LOW | Already implemented per CLAUDE.md |

### Differentiators

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| Expanded function reference with examples | 350+ functions with usage examples beyond just "key name" | MEDIUM | Add an `example` field to the JSON source files; render in help overlay |
| Module function reference (separate from `?` overlay) | CATALOG 1/2/3/4 analog in UI | MEDIUM | New panel or tab in existing `?` overlay |
| Keyboard shortcut cheat sheet panel | CLI users and GUI keyboard users need this | LOW | Static list of all `resolveKeyId` mappings |
| Contextual tooltips on hover | Hover over a key → see function description | MEDIUM | Requires key-hover state + tooltip positioning in React; tooltip data from help_entries JSON |

### Anti-Features

| Anti-Feature | Why Requested | Why Problematic | Alternative |
|--------------|---------------|-----------------|-------------|
| Full interactive tutorial (guided keystroke tour) | Best-in-class onboarding | In a calculator emulator, intercepting keystrokes mid-tutorial conflicts with normal dispatch; requires a tutorial state machine layered over the existing modal system | 3-step static RPN intro modal achieves 80% of the user value at 5% of the complexity |
| Video tutorial integration | Modern onboarding trend | Requires video hosting, bandwidth, embedding; out of scope for local-only privacy model | Link to community resources in help overlay |
| AI assistant integration | "Ask anything" appeal | Network calls violate the privacy model; deferred permanently | Search + examples cover the practical need |

### First-Run Detection

`settings.json` in `~/.hp41/` (separate from `autosave.json`) should hold:
```json
{
  "theme": "dark",
  "first_run_done": false,
  "version": "4.0"
}
```

The GUI reads this on startup via a new Tauri command `get_settings` / `save_settings`. The CLI can read the same file to track theme preference (CLI uses ratatui color schemes). The core library never touches this file.

### Dependencies

- No `hp41-core` changes.
- New Tauri command pair: `get_settings` / `save_settings`.
- New permission TOMLs in `hp41-gui/src-tauri/permissions/`.
- `help_data.ts` in GUI already loads 5 JSON pools — example fields require adding an `example?: string` field to `HelpEntry` in TypeScript and the parallel `HelpEntry` struct in `hp41-cli/src/help_data.rs`.

---

## Area 3: GUI Keyboard Parity

### Background

The CLI (`hp41-cli/src/keys.rs`) covers more keyboard paths than the GUI's `resolveKeyId()` in `App.tsx`. The gap is the primary friction point for power users who prefer keyboard over mouse. The current GUI keyboard map covers:

- F7/F8 → SST/BST
- Digits 0-9, `.`, `e` → entry
- Enter, Backspace → enter/CLx
- `+`, `-`, `*`, `/`, `%` → arithmetic
- `r`, `x`, `l`, `s`, `p`, `P` → R↓, X⟷Y, LASTX, √x, PRGM, PRX
- Shift prefix via Tab key (added per quick-task 260522-gud)
- `a`, `c`, `k` → ASIN, ACOS, ATAN
- `C`, `T`, `L`, `G`, `E`, `H`, `I`, `W`, `Y` → COS, TAN, LN, LOG, EXP, 10^X, 1/X, X², Y^X
- `u` → USER mode
- `z`, `Z`, `m`, `D`, `y`, `b`, `O`, `V` → Sigma+, Sigma-, MEAN, SDEV, YHAT, LR, CORR, CL-SIGMA
- `h`, `j`, `J` → HMS conversions
- `q`, `g` → SIN, CLREG
- `n` → CHS (or EEX-CHS)
- ALPHA mode: A-Z, 0-9, Space routes to `alpha_<X>`

The CLI also handles:
- `F5` → save state manually
- `Ctrl+W/R/D/F` → card reader operations
- Modal prompt routing for STO/RCL/ISG/DSE/SF/CF/FS?/FIX/SCI/ENG via keyboard
- XEQ by name modal (typing function name + Enter)
- GTO by name modal
- LBL prompt
- CLP prompt

**Gap assessment:** The GUI already has most CLI parity via the modal infrastructure in `pending_input.ts`. The remaining gaps are:
1. Card reader shortcuts (`Ctrl+W/R/D/F`) not wired in GUI — these invoke `dispatch_op('wdta')` / `dispatch_op('rdta')` etc., which work if exposed.
2. Some shifted function shortcuts accessible in CLI via `shifted_key_to_op` but not reaching the GUI `shiftActive` path for obscure combinations.
3. `F5` manual-save shortcut in CLI has no GUI equivalent.

### Table Stakes

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| `Ctrl+W/R/D/F` card reader shortcuts in GUI | CLI has these; GUI users expect parity | LOW | Add to `resolveKeyId` MAP in App.tsx; these are already wired as key_map entries |
| `F5` manual save shortcut in GUI | CLI has this; power users expect it | LOW | New `save_state` Tauri command (or reuse existing persistence) |
| Verify all shifted operations reachable via Tab+key | Some CLI shifted combos may not translate to GUI shiftActive path | MEDIUM | Audit KEY_DEFS.shifted coverage vs cli/keys.rs shifted_key_to_op |

### Differentiators

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| Physical keyboard shortcut reference in help overlay | Users can't discover keyboard shortcuts without this | LOW | Static table of all `resolveKeyId` bindings added to `?` overlay |
| All modal-opener shortcuts keyboard-accessible | Power users want zero mouse usage | LOW | Most already work via modal infrastructure; verify completeness |

### Anti-Features

| Anti-Feature | Why Requested | Why Problematic | Alternative |
|--------------|---------------|-----------------|-------------|
| Global (system-wide) keyboard shortcuts | "Use calculator while other apps are active" | Tauri global-shortcut plugin silently fails if another app claims the key; also undesirable to intercept keystrokes globally for a calculator | Window-focused `keydown` listener is correct scope for this app |
| Fully remappable keyboard | Power user customization | Requires a keyboard mapping UI; USER-mode ASN already covers calculator-level remapping | Document shortcuts; defer remapping UI |

### Implementation Notes

The GUI keyboard handler `resolveKeyId()` is a pure function that maps `KeyboardEvent` → string key ID. Adding card reader shortcuts is 4 lines. The `shiftActive` path in `handleKey` already applies to physical keys — the shifted variants are defined in `KEY_DEFS[].shifted`. Any gap is a missing `.shifted` entry in `Keyboard.tsx`, not an architectural issue.

Card reader commands need Tauri command exposure but the underlying `hp41-core` `CardOpRequest` drain is already wired in `commands.rs` for the click-through path.

---

## Area 4: `.raw` HP-41 Binary Program File Format

### Background — Format Specification

A `.raw` file contains the raw bytes of an HP-41 program as stored in the calculator's program memory — no header, no metadata, no checksum. This is the de facto community exchange format for HP-41 programs (thousands available at hp41.org/raw/, hpmuseum.org, etc.). The LIF (Logical Interchange Format) container is the alternative — `.raw` lives inside LIF as file type `E080` (HP-41 user-code program).

**Byte encoding (reconstructed from HP Museum Synthetic QRG, V41 Help.txt, and community forum analysis):**

The HP-41 instruction set is variable-length, 1–7 bytes per instruction. Key encoding patterns:

| Byte Range | Interpretation | Size |
|------------|----------------|------|
| `0x00–0x0F` | Local labels 0–15 and special: `0x0D` = RTN (1 byte) | 1 byte |
| `0x10–0x1C` | Miscellaneous primaries (Sigma+, 1/X, SQR, etc.) | 1 byte |
| `0x1D–0x1F` | GTO/XEQ/W with ALPHA label; followed by length nibble + chars | 2+ bytes |
| `0x20–0x2F` | Short-form RCL reg 0–15 | 1 byte |
| `0x30–0x3F` | Short-form STO reg 0–15 | 1 byte |
| `0x40–0x7F` | Various primaries: display modes, conditionals, stack ops | 1 byte |
| `0x80–0x8F` | Numeric digit entries (number literals follow as digit sequence, terminated by NULL `0x00`) | Multi-byte |
| `0x90` | RCL reg NN (2 bytes: 0x90 NN) | 2 bytes |
| `0x91` | STO reg NN (2 bytes: 0x91 NN) | 2 bytes |
| `0xA0–0xAF` | XROM: `0xAX 0xYZ` where X encodes XROM ID upper bits, YZ encodes ID lower + function | 2 bytes |
| `0xB1–0xBF` | Two-byte local GTO (0–14): `0xBN label_distance` | 2 bytes |
| `0xC0–0xCF` | GLOBAL labels and END: `0xC0` = global END; `0xC1` = local END; 3-byte structure for global labels including distance-to-previous and character count | 3+ bytes |
| `0xD0–0xDF` | Three-byte GTO: `0xDN NN distance` | 3 bytes |
| `0xE0–0xEF` | Three-byte XEQ | 3 bytes |
| `0xF0–0xFF` | Alpha string prefix: `0xFN` = N characters follow (1–15 chars) | 1+N bytes |

**Critical details:**
- **Global END** (`0xC0`): 3 bytes total. First byte `0xC0`, next two encode distance to previous global in the chain. The `0x1D`/`0x1E`/`0x1F` global-target instructions (GTO/XEQ/W with ALPHA label) are 2+ bytes: the byte itself, then a length nibble, then the ASCII characters of the label.
- **Global labels** (`0xC0–0xCF`): the top nibble `C` signals a global; the next 2 bytes encode offset. A separate preceding END marks the start of a program; global labels chain via offset links so the calculator can quickly find named programs.
- **XROM encoding**: `0xAX` where upper bits come from XROM ID; second byte `0xYZ` where `Y` = low bits of XROM ID and `Z` = function number within module. XROM IDs are restricted to 0–31; function numbers 0–63.
- **No file-level header**: a `.raw` file begins at the first instruction byte. A single-program file typically ends with the global END bytes.
- **Multiple programs**: a `.raw` file can contain multiple programs; they are separated by END markers and linked by global offsets.
- **Numeric literals**: digit sequences initiated by a byte in `0x80–0x8F`; each digit is appended; a null byte `0x00` terminates. The decimal point and EEX separator have specific encodings within this stream.

**Tagged RAW files**: A community proposal (NutEm/PC and FocalMaster) uses the END instruction's reserved fields to tag a `.raw` file as HP-41-specific. This tagging is optional and ignored by emulators — plain `.raw` files are the norm.

**LIF container**: `.raw` files can also be embedded in LIF disk images as type `E080` records. Import from LIF is a stretch goal; `.raw` extract is the primary use case.

### Table Stakes

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| Import `.raw` file — load program into `state.program` | Community standard; interop with thousands of programs | MEDIUM | Parse variable-length byte stream; map byte opcodes to `Op` variants |
| Export `.raw` file — serialize `state.program` to binary | Round-trip fidelity; share programs with hardware and other emulators | MEDIUM | Inverse map `Op` → byte(s); handle global END encoding |
| CLI: `--import-raw <file>` flag or runtime command | CLI users need this without GUI | MEDIUM | Parallels existing card reader `Ctrl+W/R` pattern |
| GUI: File dialog for `.raw` import/export | GUI users expect native file picker | LOW | Tauri v2 `dialog` plugin for open/save file |
| Error handling: unknown bytes → informative error | Malformed files are common in community archives | LOW | Return `Err` with byte offset and value |

### Differentiators

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| Multi-program `.raw` file support (import all programs) | Most community `.raw` files contain multiple programs | MEDIUM | Parse the global-label chain; load each named segment |
| Export with correct global END chaining | Files that work on real HP-41 hardware and V41 | HIGH | The offset chain between globals must be computed correctly |
| Decompile `.raw` to text listing (dry-run import) | Debug aid; verify file before loading | MEDIUM | Emit a text listing to CLI output or GUI print panel |
| Round-trip test suite | CI guard that import→export→import is lossless | MEDIUM | Mirrors existing card reader SHA-256 round-trip test |

### Anti-Features

| Anti-Feature | Why Requested | Why Problematic | Alternative |
|--------------|---------------|-----------------|-------------|
| LIF disk image mounting | Full HP-IL compatibility | LIF image format has 256-byte record structure, directory parsing, multi-file layout — a separate project. LIF extraction tools (LIFUTIL, HP41UC) already exist | Import `.raw` files extracted from LIF images |
| Full XROM byte resolution for all 5 modules | Import programs that use XROM functions | XROM byte encoding depends on XROM ID + function number; the existing `xrom_resolve` already knows IDs. The import path just needs to emit `Op::XromCall { id, fn_num }` and let the resolver handle it | Use existing resolver; no special handling needed |
| HP-41 compiled ROM (`.mod`) import | Module interop | `.mod` files are 4096-byte or 8192-byte ROM images containing M-CODE; importing M-CODE bytecode is cycle-accurate emulation territory, not behavioral | Out of scope permanently |

### Implementation Notes — Op Mapping

The import direction maps bytes → `Op` variants. The challenge is the XROM encoding: `0xAX 0xYZ` encodes `(xrom_id, fn_num)` but the corresponding `Op` variant depends on which module is loaded. The correct approach:

1. Parse `0xAX 0xYZ` → extract `xrom_id = ((X & 0x1) << 4) | (Y >> 4)` and `fn_num = ((Y & 0xF) << 2) | (Z >> 6)` (approximate — exact nibble layout needs verification against the HP-41 Synthetic QRG byte table).
2. Emit `Op::XromDispatch { xrom_id, fn_num }` — a new Op variant that defers resolution to runtime via `xrom_resolve`. This avoids needing to know which module is loaded at import time.
3. Alternatively, eagerly resolve via `xrom_resolve` at import time — simpler but fails if the relevant module isn't loaded.

The `Op::XromDispatch` deferred-resolution approach is the cleaner design and avoids the 4-way exhaustive-match obligation for every known XROM function — only one new variant is needed.

**Confidence note:** The exact nibble layout of XROM byte encoding is MEDIUM confidence — the general structure is confirmed by multiple sources but the exact bit-field positions need verification against the HP Museum byte table (currently 403-blocked) or the HP-41 Synthetic QRG PDF (binary content, not web-readable).

### Dependencies

- **New `hp41-core` function**: `fn import_raw(bytes: &[u8]) -> Result<Vec<Op>, HpError>` and `fn export_raw(program: &[Op]) -> Result<Vec<u8>, HpError>` in a new `hp41-core/src/ops/raw_format.rs`.
- **4-way invariant**: `Op::XromDispatch` (if introduced) must land in `dispatch()`, `execute_op()`, and both `prgm_display.rs` files.
- **CLI**: new command or `handle_key` action for import/export — mirrors the card reader `Ctrl+W/R` shortcuts.
- **GUI**: Tauri v2 `dialog` plugin permission + new `import_raw_file` and `export_raw_file` Tauri commands.
- **`serde(skip)`** on any transient state added for raw parsing.

---

## Area 5: Extended Memory (X-MEM) Model

### Background — Hardware Architecture

The HP-41CX and HP-41C/CV with Extended Memory modules use X-MEM as a secondary file store. The data model, reconstructed from the V41 Help.txt internal dump format, Wikipedia, and HP Museum forum discussions:

**Capacity:**
- HP-41CX built-in Extended Functions module: 124 registers of X-MEM
- Each Extended Memory module adds 238 registers
- Maximum with 2 modules: 124 + 476 = 600 registers
- Each register = 7 bytes (HP-41 word size)

**File organization:**
X-MEM is a singly-linked list of files. Each file begins with:
- **Register 0**: Name (up to 6 HP-41 FOCAL characters packed in register-width encoding)
- **Register 1**: Type byte + length field (number of registers the file occupies)
- **Data registers**: The file's payload
- **Last register**: Checksum

**File types (hardware-defined):**
| Type Byte | Type Name | Contents |
|-----------|-----------|----------|
| `'P'` (0x50) | PRGM | HP-41 program (same byte encoding as `.raw`) |
| `'D'` (0x44) | DATA | Numbered data registers (R00–RNN dump) |
| `'S'` (0x53) | STAT | Sigma-accumulation registers |
| `'A'` (0x41) | ASCII | Text file (Text Editor) |
| `'K'` (0x4B) | KEY | User key assignments |
| `'X'` (0x58) | XMEM | Sub-allocated extended memory block |

**File chain termination:** The last entry in the chain is `FF FF FF FF FF FF FF` (7 bytes of 0xFF).

**Link structure (from V41 Help.txt):**
```
040: 00 00 10 00 00 00 BF  Link ID=0BF Next=000
```
The third byte appears to encode the file size (number of registers); the last byte encodes the link to the next file's register address. Exact nibble layout is MEDIUM confidence — confirmed structurally but bit-field positions need verification.

**Operations:**
- `EMDIR` (Extended Memory DIRectory): prints a catalog of all X-MEM files to the print buffer. Equivalent to `CATALOG 4` on HP-41CX.
- `EMROOM` (Extended Memory ROOM): returns the number of free registers remaining in X-MEM. Result in X.
- `EMREG` (Extended Memory REGister): stores/recalls data register sets to/from X-MEM files. Similar to the card reader data register operations but targeting X-MEM.
- `SAVEP <name>` / `GETP <name>`: save/retrieve a program by name to/from X-MEM.
- `SAVED <name>` / `GETD <name>`: save/retrieve data registers to/from X-MEM.
- `PURFL <name>` / `PURALL`: purge named file / purge all X-MEM.
- `FLSIZE <name>`: return size of named file.
- `XROOM`: same as EMROOM (alias).
- `CATALOG 4`: list X-MEM files (equivalent to EMDIR).

**Relationship to Advantage Pac ADVMTRX:** The Advantage Pac's named-matrix model (`adv_matrices: Vec<AdvMatrix>` on `CalcState`) was intentionally designed as a minimal in-memory store that does NOT implement full X-MEM (see v3.3 Anti-Features in the existing FEATURES.md). The v4.0 X-MEM model can reuse `adv_matrices` for the matrix storage side, but also needs to support PRGM and DATA file types for the core EMDIR/EMROOM/EMREG operations that HP-41CX users expect.

### Table Stakes

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| `EMDIR` — list all X-MEM files to print buffer | HP-41CX users consider this basic; `CATALOG 4` equivalent | MEDIUM | Iterate `xmem_files` Vec; format name, type, size |
| `EMROOM` — return free register count | Used in programs to check space before writing | LOW | Simple subtraction: total capacity - used registers |
| `SAVEP <name>` / `GETP <name>` — program file I/O | Save a program by name; retrieve by name | HIGH | `export_raw`-style serialization into X-MEM file entry |
| `SAVED <name>` / `GETD <name>` — data register file I/O | Save/retrieve numbered register set | MEDIUM | Serialize `state.regs[0..N]` into DATA file entry |
| `PURFL <name>` / `PURALL` — delete X-MEM files | File lifecycle management | LOW | Remove from `xmem_files` Vec |
| `FLSIZE <name>` — query file size | Used by programs to validate X-MEM contents | LOW | O(1) lookup |

### Differentiators

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| X-MEM persistence across save/load | Programs stored in X-MEM survive app restart | MEDIUM | `xmem_files: Vec<XMemFile>` on CalcState with `#[serde(default)]` |
| `CATALOG 4` wired to `EMDIR` | Hardware-authentic | LOW | New arm in `dispatch()` for `Op::Catalog(4)` |
| X-MEM file export to `.raw` | Bridge between X-MEM PRGM files and community `.raw` exchange | MEDIUM | Reuse `export_raw()` function; apply to PRGM file payload |
| X-MEM file import from `.raw` | Load community programs directly into named X-MEM slot | MEDIUM | Reuse `import_raw()` function; wrap in X-MEM file entry |

### Anti-Features

| Anti-Feature | Why Requested | Why Problematic | Alternative |
|--------------|---------------|-----------------|-------------|
| Full 600-register fidelity (hardware byte-level simulation) | Maximum hardware accuracy | The linked-list register layout with exact nibble-field offsets is a V41-level implementation detail; behavioral emulation just needs a Vec of files with name, type, payload | Model as `Vec<XMemFile { name, kind, data }>` on CalcState |
| STAT and ASCII file types | Completeness | STAT files are just Sigma-register dumps; ASCII/Text Editor is a separate large feature (Text Editor has its own keyboard mode). Defer both | PRGM and DATA files cover the core use case |
| KEY file type (ASN storage in X-MEM) | Completeness | User key assignments are already in `state.assignments: BTreeMap<u8, String>` — persisting them redundantly in X-MEM adds complexity for minimal gain | Already covered by JSON autosave |
| HP-IL device compatibility (write X-MEM to IL disk) | Hardware enthusiasts | HP-IL emulation is permanently out of scope | `.raw` export bridges to physical media via PC tools |

### Data Model

New type in `hp41-core/src/state.rs` (or a dedicated module):

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum XMemFileKind {
    Prgm,   // program bytecode (Op sequence)
    Data,   // numbered registers
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XMemFile {
    pub name: String,          // up to 6 chars, HP-41 ALPHA subset
    pub kind: XMemFileKind,
    pub program: Vec<Op>,      // populated when kind == Prgm
    pub registers: Vec<HpNum>, // populated when kind == Data
}
```

Add to `CalcState`:
```rust
#[serde(default)]
pub xmem_files: Vec<XMemFile>,
```

`EMROOM` is computed on-the-fly: `600 - xmem_files.iter().map(|f| f.register_count()).sum::<usize>()`.

### Dependencies

- **4-way invariant**: New `Op` variants for each XROM-sourced X-MEM operation (EMDIR, EMROOM, SAVEP, GETP, SAVED, GETD, PURFL, PURALL, FLSIZE) must land in all four exhaustive-match locations.
- **`.raw` area dependency**: SAVEP/GETP rely on the `.raw` import/export functions from Area 4. Area 4 should be implemented first.
- **CATALOG 4**: The existing `Op::Catalog(n)` dispatch already handles 1–4; adding case 4 → EMDIR is a new arm in `dispatch()`.
- **serde backward-compat**: `xmem_files` needs `#[serde(default)]`; v3.3 save files load with an empty Vec.

---

## Feature Dependencies (Cross-Area)

```
Skin Themes (Area 1)
    └──requires──> CSS variable extraction from App.css (refactor)
    └──requires──> settings.json persistence (new Tauri command)
    └──independent of hp41-core

Onboarding / Reference (Area 2)
    └──requires──> settings.json persistence (shared with Area 1)
    └──enhances──> existing ? overlay (additive, no breakage)
    └──independent of hp41-core

GUI Keyboard Parity (Area 3)
    └──requires──> verify card reader commands wired (already in hp41-core)
    └──independent of Areas 1, 2

.raw File I/O (Area 4)
    └──requires──> new raw_format.rs in hp41-core
    └──requires──> Op::XromDispatch variant (if deferred-resolution chosen)
    └──requires──> Tauri dialog plugin for GUI
    └──blocks──> X-MEM SAVEP/GETP (Area 5 depends on this)

X-MEM Model (Area 5)
    └──requires──> .raw import/export (Area 4) — SAVEP/GETP delegate to raw codec
    └──requires──> XMemFile type + xmem_files field on CalcState
    └──requires──> new Op variants (4-way invariant)
    └──independent of Areas 1, 2, 3
```

### Dependency Notes

- **Areas 1+2 share** `settings.json` — implement `get_settings` / `save_settings` once, use for both theme and first-run flag.
- **Area 4 blocks Area 5** for PRGM file I/O: SAVEP serializes a program to `.raw` bytes; GETP deserializes. If Area 4 ships first, Area 5 PRGM support comes cheaply.
- **Area 3 is independent** — it is purely frontend changes to `App.tsx` with no core changes.

---

## MVP Definition

### Phase Structure Recommendation

**Phase A — Skin Themes (Areas 1) + Settings infrastructure:**
- Extract ~22 CSS custom properties from App.css
- Implement 4 built-in themes (dark, light, beige, hi-contrast)
- Theme switcher in UI (dropdown or button row)
- `settings.json` persistence (shared infrastructure for Phase B)

**Phase B — Onboarding + GUI Keyboard Parity (Areas 2 + 3):**
- First-run welcome modal with RPN quick-start (3-step)
- Physical keyboard shortcut reference panel in `?` overlay
- Card reader shortcuts (`Ctrl+W/R/D/F`) in GUI
- `F5` manual save in GUI

**Phase C — .raw File I/O (Area 4):**
- `import_raw` / `export_raw` functions in `hp41-core`
- CLI: `Ctrl+I`/`Ctrl+E` or `:raw` command
- GUI: file dialog import/export buttons

**Phase D — X-MEM Model (Area 5):**
- `XMemFile` type + `xmem_files` on `CalcState`
- `EMDIR`, `EMROOM`, `SAVEP`/`GETP`, `SAVED`/`GETD`, `PURFL`/`PURALL`, `FLSIZE`
- `CATALOG 4` → EMDIR
- X-MEM ↔ `.raw` bridge

**Phase E — Test Hardening + Quality Gates:**
- `.raw` round-trip tests
- Theme visual regression (manual)
- X-MEM backward-compat (load v3.3 save files)
- Coverage gate maintenance

---

## Feature Prioritization Matrix

| Feature | User Value | Implementation Cost | Priority |
|---------|------------|---------------------|----------|
| Skin themes (4 presets) | HIGH | LOW | P1 |
| Theme persistence | HIGH | LOW | P1 |
| First-run RPN onboarding modal | HIGH | LOW | P1 |
| GUI keyboard parity (card reader + F5) | MEDIUM | LOW | P1 |
| `.raw` file import | HIGH | MEDIUM | P1 |
| `.raw` file export | HIGH | MEDIUM | P1 |
| settings.json persistence | HIGH | LOW | P1 |
| `?` overlay keyboard shortcut reference | MEDIUM | LOW | P2 |
| X-MEM EMDIR / EMROOM | MEDIUM | MEDIUM | P2 |
| X-MEM SAVEP / GETP (program files) | MEDIUM | HIGH | P2 |
| X-MEM SAVED / GETD (data files) | MEDIUM | MEDIUM | P2 |
| X-MEM purge + FLSIZE | MEDIUM | LOW | P2 |
| Function reference examples in help | LOW | MEDIUM | P3 |
| Contextual key hover tooltips | LOW | MEDIUM | P3 |
| X-MEM STAT / ASCII file types | LOW | HIGH | P3 |
| External skin file loading | LOW | HIGH | P3 |

---

## Competitor Feature Analysis

| Feature | V41 (Windows) | Free42/Plus42 | cs-41 (macOS) | Our Approach |
|---------|---------------|---------------|----------------|--------------|
| Themes | None | External .gif bitmap skins | None documented | 4 built-in CSS presets |
| Onboarding | Plain Help.txt | None | None | First-run modal + shortcut reference |
| `.raw` import | Yes (core feature) | No (different format) | Unknown | Yes — core community interop |
| `.raw` export | Yes | No | Unknown | Yes |
| X-MEM emulation | Yes (full, hardware-accurate) | N/A (HP-42S has no X-MEM) | Unknown | Behavioral: PRGM + DATA file types |
| Physical keyboard | Partial | Full (desktop apps) | Unknown | Parity gap closed |

---

## Sources

- V41 Help.txt (X-MEM file structure, link chain format): [hp.giesselink.com/V41/Help.txt](https://hp.giesselink.com/V41/Help.txt) — MEDIUM confidence (format reconstructed from internal dump examples)
- HP-41 byte encoding (instruction set): [HP-41 Synthetic QRG community documentation](https://literature.hpcalc.org/community/hp41-synthetic-qrg.pdf) — MEDIUM confidence (PDF binary, content reconstructed from search results cross-referencing)
- Tagged RAW Files discussion (plain vs tagged format): [hp41.org forum thread 610](https://forum.hp41.org/viewtopic.php?f=21&t=610) — HIGH confidence
- HP-41 instruction encoding (XROM, GTO, global labels): [HP Museum Synthetic Programming + community search results](https://www.hpmuseum.org/prog/synth41.htm) — MEDIUM confidence (byte table is 403-blocked; reconstructed from multiple secondary sources)
- HP-41CX Extended Memory overview: [Wikipedia HP-41 extension module](https://en.wikipedia.org/wiki/HP-41_extension_module) — MEDIUM confidence
- HP-41 X-MEM file types discussion: [HP Museum forum thread 13684](https://www.hpmuseum.org/forum/thread-13684.html) — 403-blocked, information from search result excerpts — LOW-MEDIUM confidence
- Free42 skin format: [thomasokken.com/free42/skins/README.html](https://thomasokken.com/free42/skins/README.html) — HIGH confidence
- CSS custom property theming patterns: [CSS-Tricks light-dark()](https://css-tricks.com/almanac/functions/l/light-dark/), [design.dev dark mode guide](https://design.dev/guides/dark-mode-css/) — HIGH confidence
- Tauri v2 theme switching: [GitHub Tauri discussion #13472](https://github.com/tauri-apps/tauri/discussions/13472) — HIGH confidence
- LIF file type E080 (HP-41 program): [HP Museum LIF file types thread 2192](https://www.hpmuseum.org/forum/thread-2192.html) — 403-blocked; confirmed via search result excerpts — MEDIUM confidence
- HP-41 community archive (.raw files): [hp41.org/raw/](http://hp41.org/raw/) — HIGH confidence (empirical: thousands of .raw files available)

---

*Feature research for: v4.0 Platform Maturity — HP-41 calculator emulator*
*Researched: 2026-05-27*
