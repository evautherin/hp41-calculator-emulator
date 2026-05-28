# Pitfalls: v4.0 Platform Maturity

**Domain:** Adding theming, onboarding, GUI keyboard parity, `.raw` file I/O, and Extended Memory to a mature 100K+ LOC Rust/Tauri/React HP-41 emulator
**Researched:** 2026-05-27
**Confidence:** HIGH (codebase-derived) / MEDIUM (X-MEM hardware spec, community format edge cases)

**Scope:** Pitfalls SPECIFIC to v4.0 features in THIS codebase. Prior pitfalls documented in v3.x PITFALLS.md files (P1–P50) are not repeated unless they have a new v4.0 failure mode.

---

## Summary

Eight pitfall clusters dominate the v4.0 risk surface:

1. **CSS theming breaks SVG key animations** (P51) — CRITICAL. `transform-box: fill-box` on `.key` is a load-bearing invariant; any theme injection that resets `transform-box` will break press animations on every key.

2. **`.raw` multi-program file rejection hides community programs** (P52) — CRITICAL. The existing decoder rejects trailing bytes after the END marker — this is correct for single-program files but breaks multi-program `.raw` archives (common in the HP-41 community). Silently loading only the first program would be worse.

3. **CalcState X-MEM fields missing `#[serde(default)]` breaks all existing save files** (P53) — CRITICAL. Every new persistent field in `CalcState` requires `#[serde(default)]` or v1.0–v3.3 save files will panic on load. The X-MEM model adds several fields.

4. **GUI keyboard parity creates undiscovered `resolveKeyId` gaps** (P54) — HIGH. The GUI `resolveKeyId` in `App.tsx` is a handwritten `Record<string, string>` that is NOT mechanically derived from `hp41-cli/src/keys.rs`. Gaps exist; parity audit must be systematic.

5. **Theme CSS custom properties not reaching SVG `<defs>` gradients** (P55) — HIGH. SVG gradient `stopColor` values inside `<defs>` are set as JSX props (compile-time literals in `Keyboard.tsx`). CSS variables injected on the document body do NOT propagate into SVG `<defs>` stop elements — those need React state or inline style prop injection.

6. **X-MEM as its own `Vec<XmemFile>` field isolation from main registers** (P56) — HIGH. The Advantage Pac named-matrix isolation lesson (D-43.5) applies directly: X-MEM must never share indexing with `state.regs` or `state.adv_matrices`. A new `xmem_files: Vec<XmemFile>` field is the right model; sharing address space with `regs` would cause silent corruption.

7. **Onboarding state persisted across sessions causes first-run to never re-trigger** (P57) — MEDIUM. If the "has seen onboarding" flag lives in `CalcState` (and thus `autosave.json`), the onboarding never shows after the first clear. If it lives outside `CalcState`, the shared `autosave.json` path is irrelevant — it must go somewhere else (OS config dir).

8. **`.raw` XROM opcode encoding for v3.x modules** (P58) — MEDIUM. The existing `raw.rs` encoder returns `HpError::CardData` for any Op outside its subset. If users try to export programs containing XROM calls to Math Pac / Stat Pac / Time / Advantage Pac ops, the export silently fails. The XROM two-byte encoding (`0xAx <rom_id_6bit>:<fn_6bit>`) is well-defined but not yet implemented.

---

## Critical Pitfalls

### Pitfall 51: CSS Theming Breaks SVG Key Animation Invariant

**What goes wrong:**
The `.key` CSS class has two load-bearing declarations:
```css
transform-box: fill-box;    /* REQUIRED for SVG transform-origin */
transform-origin: center;
transition: transform 80ms ease-out;
```
Any CSS theme that replaces `.key` styles or uses a CSS reset at a higher specificity level will clear `transform-box: fill-box`. When this happens, `transform-origin: center` resolves relative to the SVG viewport origin (top-left of the `<svg>` element), not the key's own bounding box. Keys will appear to scale from the top-left corner of the entire keyboard rather than from the center of the pressed key — visually broken on every keypress.

**Why it happens:**
Developers implementing themes add a new stylesheet or modify `App.css` with CSS custom properties, then test by clicking a few keys. The press animation appears "slightly off" but not catastrophically broken. It only becomes obvious when testing keys at the extreme bottom-right of the keyboard (ENTER, +, ×, ÷) where the origin offset is maximally visible.

A CSS reset (`* { transform-box: unset; }`) or a theme stylesheet that redeclares `.key` without `transform-box: fill-box` silently removes it.

**How to avoid:**
- The `transform-box: fill-box` declaration must appear in EVERY theme variant, not just in a base stylesheet that themes override.
- Use CSS custom properties (`--key-bg-color`, `--shift-color`, etc.) ONLY for color/opacity values — never touch `transform-box`, `transform-origin`, or `transition`.
- Add a dedicated Vitest/browser test that asserts `getComputedStyle(keyElement).transformBox === 'fill-box'` for at least one key element across all theme variants.
- Treat `.key { transform-box: fill-box; transform-origin: center; }` as invariant boilerplate that appears in a `_keys-animation.css` file imported before any theme, with `!important` if necessary.

**Warning signs:**
- Key press animation scales from a corner instead of center.
- `getComputedStyle`.`transformBox` returns anything other than `'fill-box'` in devtools.
- CI Vitest passes but manual test shows broken animation (animation correctness is not automatically tested in current test suite).

**Phase to address:** Theme Phase (first phase introducing theme switching)

---

### Pitfall 52: `.raw` Multi-Program File Rejection Silently Discards Community Content

**What goes wrong:**
The existing `decode_program()` in `hp41-core/src/cardreader/raw.rs` explicitly rejects trailing bytes after the END marker:
```rust
if i + END_MARKER.len() < bytes.len() {
    return Err(HpError::CardData(format!(
        "trailing bytes after END marker: {} extra byte(s)",
        ...
    )));
}
```
This is correct for single-program files. However, HP-41 community `.raw` archives frequently contain multiple programs concatenated into one file (the V41 emulator's "Get" command extracts them all). A user who downloads a `.raw` archive of 5 programs and tries to import it will get `CardData` error and zero programs imported. The error message mentions "trailing bytes" which provides no actionable guidance.

**Why it happens:**
The HP-41 `.raw` format has no official multi-program spec. The current implementation was designed for round-trip fidelity with a known-single-program encoding. The "trailing bytes = error" invariant was intentionally strict to prevent silently loading only the first of several concatenated programs.

**How to avoid:**
Two acceptable strategies:
1. **Single-program only with clear error:** Keep the strict rejection, but change the error message to say: "File contains multiple programs — import only handles single-program `.raw` files. Use a tool like HP41UC to split the archive." This is safe and honest.
2. **Multi-program import:** Add a `decode_all_programs()` function that repeatedly calls the single-program decoder, stopping at each END marker, and returns a `Vec<Vec<Op>>` with one entry per program. Import UI lets the user choose which program(s) to load.

The multi-program strategy is better UX but adds complexity. The key invariant is: NEVER silently import only the first program. Either import all programs or clearly reject multi-program files.

**Warning signs:**
- User reports "CardData: trailing bytes" error when importing `.raw` files from hp41.org.
- Any `.raw` file over ~200 bytes that isn't a single program with a complex numeric constant sequence.

**Phase to address:** `.raw` Import/Export Phase

---

### Pitfall 53: New X-MEM `CalcState` Fields Without `#[serde(default)]` Breaks All Existing Save Files

**What goes wrong:**
The serde backward-compatibility invariant (D-07 in CLAUDE.md) requires every new `CalcState` field to carry `#[serde(default)]`. If even one X-MEM field is added without this attribute, `serde_json` will return a deserialization error when any v1.0–v3.3 save file is loaded. The autosave is shared between CLI and GUI — both will fail on startup with a cryptic JSON error.

The v3.x history shows 5 distinct serde shape patterns in `CalcState`:
1. `#[serde(default)]` — most fields: persist across save/load, default on old saves
2. `#[serde(default, skip)]` — transient fields: never persisted, default on load
3. `#[serde(default = "default_fn")]` — fields with non-trivial defaults (e.g. `xrom_modules`, `cancel_requested`)
4. `#[serde(default)]` WITHOUT `skip` on semantically persistent fields (only `rand_seed` and `adv_tvm_state`) — see P20 from v3.1

For X-MEM: the directory (`xmem_files: Vec<XmemFile>`), current filename, EMROOM count, and any display state all need the correct annotation.

**Why it happens:**
X-MEM adds multiple new fields at once. It is easy to add one correctly and then forget `#[serde(default)]` on a subsequent field added during the same PR, especially for the transient fields (current directory cursor, display mode).

**How to avoid:**
- Add a `CalcState` backward-compat test that loads a v3.3 save file fixture (literal JSON string in `tests/`) and asserts it deserializes without error. This test must run in CI and must fail if any field missing `#[serde(default)]` is added.
- Review pattern: every new field in `CalcState` requires a code review sign-off on its serde annotation matching the appropriate pattern from the list above.
- Add `migrate_after_load()` handling for X-MEM initialization (same pattern as stopwatch freeze on load in v3.2).

**Warning signs:**
- `cargo test` passes but `just integration-test` fails with JSON parse error against a saved fixture.
- User reports "Failed to load state" on startup after upgrading from v3.3.
- Any new `CalcState` field added in a commit that lacks `#[serde(default)]` in its diff.

**Phase to address:** X-MEM Core Phase (first phase adding X-MEM fields to `CalcState`)

---

## High-Priority Pitfalls

### Pitfall 54: GUI `resolveKeyId` Has Undiscovered Gaps vs CLI `key_to_op`

**What goes wrong:**
The GUI `resolveKeyId` function in `App.tsx` (lines 108–161) is a handwritten `Record<string, string>` MAP. It was initially created to mirror `hp41-cli/src/keys.rs::key_to_op()` but the two have never been audited side by side since v2.2. Every v3.x milestone added new CLI keys without necessarily wiring them in the GUI physical-keyboard path.

A "GUI keyboard parity" phase that only audits the on-screen click path (which goes through `KEY_DEFS` → `key_map.rs::resolve`) will miss gaps in the physical keyboard path (`resolveKeyId`). The physical keyboard is the fast path for power users.

Specific known gaps from reading the current code:
- CLI `key_to_op` maps: `'q'→sin`, `'s'→sqrt`, `'C'→cos`, `'T'→tan`, `'L'→ln`, `'G'→log`, `'E'→exp`, `'H'→tenpow`, `'I'→recip`, `'W'→sq`, `'Y'→ypow` — these ARE in `resolveKeyId`.
- CLI `shifted_key_to_op` maps conditional tests via `f` prefix — GUI maps this through `shiftActive` + `KEY_DEFS.shifted` — functional parity but different mechanism.
- `'h'→hms_to_h`, `'j'→hms_add`, `'J'→hms_sub` — these ARE in `resolveKeyId` but have no corresponding GUI on-screen path (no keycap labeled HMS). Parity for these is physical-keyboard-only which is likely intentional.
- Any new keys added during GUI keyboard parity work that go into `KEY_DEFS` but not into `resolveKeyId` will be silently unreachable from physical keyboard.

**How to avoid:**
- Extract the authoritative list from `key_to_op` and `shifted_key_to_op` in `hp41-cli/src/keys.rs` as a reference table.
- Add a TypeScript test (`App.test.tsx` or new file) that calls `resolveKeyId` for every key in the reference table and asserts a non-null result.
- The existing `key_coverage.rs` test in `hp41-cli/tests/` tests CLI key coverage but has no GUI equivalent — add a `key_coverage.test.tsx` counterpart.
- When adding a new key binding: update `key_to_op` (CLI), `KEY_DEFS` (GUI on-screen), AND `resolveKeyId` MAP (GUI physical). Make them a single diff block in the same commit.

**Warning signs:**
- Physical keyboard shortcut works in CLI but not in GUI after a parity work phase.
- `resolveKeyId` returns `null` for a key that `key_to_op` handles.
- A new key added only to `KEY_DEFS` without updating `resolveKeyId`.

**Phase to address:** GUI Keyboard Parity Phase

---

### Pitfall 55: Theme CSS Variables Do Not Propagate Into SVG `<defs>` Gradient Stops

**What goes wrong:**
`Keyboard.tsx` defines gradient colors as JSX inline props at compile time:
```tsx
<stop offset="0%"   stopColor="#1a1a1a" />    // body gradient
<stop offset="0%"   stopColor="#303030" />    // key cap gradient
<stop offset="0%"   stopColor="#d68a1c" />    // shift key color
```
These are JSX props that React compiles to SVG attribute strings. They are NOT CSS properties — CSS custom properties (`--body-bg: #1a1a1a`) set on `:root` or `[data-theme]` do NOT reach them.

A theming implementation that puts colors in CSS variables on the document body and expects SVG gradients to inherit them will produce a partial result: element colors outside `<defs>` that use `fill="var(--color)"` will work; gradient stop colors inside `<defs>` will stay hardcoded at their compile-time values.

**Why it happens:**
The CSS custom property mental model is that variables cascade through the DOM. This is true for CSS properties (`fill`, `stroke`, `color`, `background`) but NOT for SVG presentation attributes set as element attributes (including `stopColor` in `<defs>`). The two are distinct inheritance mechanisms.

**How to avoid:**
Three options (ordered by implementation complexity):
1. **Theme via React props:** Pass a `theme: ThemeConfig` prop down to `<Keyboard>` and use it to compute gradient stop colors at render time. Requires modifying `Keyboard.tsx` to accept theme props and threading state from theme selector down to keyboard.
2. **CSS `currentColor` trick:** Change all gradient stops to `stopColor="currentColor"` and control the color via CSS `color:` property on ancestor elements. This works for single-color gradients but is awkward for multi-stop gradients with different semantic colors.
3. **`<defs>` injection via `useEffect`:** After render, use `document.getElementById` to locate `<stop>` elements and set their `stopColor` attribute imperatively. Fragile — React may re-render and undo the mutation.

**Recommended approach:** Option 1. Create a `THEMES` constant object keyed by theme name, pass `theme` as a prop to `<Keyboard>`, and map theme keys to gradient stop values. This is the only approach that is React-idiomatic, type-safe, and survives re-renders.

**Warning signs:**
- Changing `--body-bg` CSS variable has no effect on the calculator body gradient.
- Theme toggle works for text elements but keyboard body/key gradients stay dark.
- Chrome devtools shows `stopColor` attribute still contains the hex literal value after theme change.

**Phase to address:** Theme Phase (same phase as P51)

---

### Pitfall 56: X-MEM Register File Must Be Isolated From `state.regs`

**What goes wrong:**
The HP-41CX Extended Memory is a separate register file (up to 600 registers in hardware). Emulators that implement X-MEM as an extension of `state.regs` (using indices 320–919) will corrupt programs that use `RCL 300` (a valid main-memory register in the extended 319-register model) if the X-MEM data occupies overlapping address space.

The Advantage Pac lesson is directly applicable: D-43.5 established that `adv_matrices` must NEVER touch `state.matrix_dim` or `state.matrix_active_reg`. The same principle applies to X-MEM: it must be a completely separate field.

Concrete consequence: if `state.regs` is extended to 919 elements and X-MEM files are mapped at indices 320+, then `CLREG` (which zeros all registers) will also clear all X-MEM content. On real HP-41CX hardware, `CLREG` only affects main memory registers.

**How to avoid:**
- X-MEM must live in a dedicated `xmem_files: Vec<XmemFile>` field in `CalcState`, never in `state.regs`.
- `XmemFile` struct: `{ name: String, data: Vec<HpValue> }` — name is the ALPHA register key, data is the file's register content.
- `EMDIR` op reads from `xmem_files` len and names, not from `state.regs`.
- `EMROOM` returns available capacity without touching `state.regs`.
- `EMREG` (register file access) uses `xmem_files[current].data[i]`, not `state.regs[i + offset]`.
- Add D-51.x decision record documenting the isolation invariant before any X-MEM code is written.

**Warning signs:**
- `CLREG` clears X-MEM file data.
- `RCL 319` in a program produces different results depending on how many X-MEM files exist.
- `EMREG` implementation code touching `state.regs`.

**Phase to address:** X-MEM Core Phase

---

## Moderate Pitfalls

### Pitfall 57: "Has Seen Onboarding" Flag Stored in Wrong Location

**What goes wrong:**
Two wrong placements:
1. **In `CalcState` (and thus `~/.hp41/autosave.json`):** The onboarding will never show again after first run regardless of whether the user completed it, because the flag travels with the calculator state and is overwritten on every save. A user who clears their save file to reset the calculator expects onboarding to reappear — it won't.
2. **In `localStorage` (GUI only):** CLI users get onboarding every time (no localStorage equivalent); GUI users see it once. Inconsistency across frontends.

**How to avoid:**
The `"has seen onboarding"` flag belongs in the OS config directory: `~/.config/hp41/prefs.json` (Linux), `~/Library/Preferences/ch.talent-factory.hp41.plist` (macOS), or equivalent via `dirs::config_dir()`. This is separate from the calculator state in `~/.hp41/autosave.json`. The GUI should use Tauri's `tauri-plugin-store` or a simple Tauri command that reads/writes this preferences file.

For CLI, onboarding (if implemented) should similarly read from the OS config dir, not `autosave.json`.

**Warning signs:**
- Resetting calculator state (deleting autosave.json) does not re-trigger onboarding.
- Onboarding shows on every CLI startup but never again in GUI (or vice versa).

**Phase to address:** Onboarding Phase

---

### Pitfall 58: `.raw` XROM Encoding Missing for v3.x Module Ops

**What goes wrong:**
The existing `raw.rs` encoder covers ~25 ops (arithmetic, stack, basic math, labels, synthetic). When a user tries to export a program that includes an XROM call — e.g., `XEQ "QUAD"` (Math Pac I) or `XEQ "STAT"` (Stat 1) — the encoder returns:
```
HpError::CardData("op cannot be encoded in the .raw subset: Xeq(\"QUAD\")")
```
This is not a bug per se (the error is explicit) but it's a significant UX limitation. Most programs written by v3.x users will include XROM calls. Export becomes near-useless for the target audience.

The XROM encoding is documented: two bytes, first byte `0xAx` where `x` is the ROM ID (0–31), second byte is the function number (0–63). This is `0xA0 + (rom_id & 0x1F)` for the first byte. However, the emulator uses `Op::Xeq("NAME")` internally — mapping names to ROM_ID + function_number requires consulting the 5 JSON function tables to find which XROM module and function index a given name maps to.

**How to avoid:**
- Build a name→(xrom_id, fn_index) lookup table from the 5 JSON pools in `hp41-core` (or `hp41-cli/src/help_data.rs`) at encode time.
- Alternatively, add an `Op::XromCall { module_id: u8, fn_index: u8 }` variant alongside `Op::Xeq(String)` for the binary-encoded case, with the encoder emitting the two-byte form.
- At minimum: change the error message to explain the XROM encoding limitation and point users to HP41UC for conversion.

**Warning signs:**
- Any `XEQ "name"` for a Math/Stat/Time/Advantage op fails export.
- User reports "raw export fails for any useful program."

**Phase to address:** `.raw` Import/Export Phase

---

### Pitfall 59: Theme Persistence Shared-State Race Between CLI and GUI

**What goes wrong:**
If theme selection is persisted (so it survives app restarts), it must be stored somewhere. If it is stored in `autosave.json` (shared between CLI and GUI), then:
- CLI reads the file, ignores the theme field (CLI uses terminal colors), and saves — no problem.
- But the `CalcState` serde contract requires `#[serde(default)]` on any new field; the CLI binary won't know about GUI theme fields and must tolerate them (P53 applies).
- If theme is stored in GUI-specific config only, CLI never touches it — cleaner.

The deeper issue: theme is not calculator state. It is a presentation preference. Putting it in `CalcState` conflates UI preferences with emulator state, making the JSON file harder to reason about.

**How to avoid:**
Store theme preference in the GUI-specific prefs file (same location as P57 onboarding flag). Use Tauri's plugin-store or a dedicated prefs command. Do NOT add a `theme: ThemeKind` field to `CalcState`.

**Warning signs:**
- `autosave.json` contains a `"theme"` key.
- CLI binary fails to load a save file written by GUI because of unknown `theme` field (if `#[serde(default)]` was forgotten — same failure mode as P53).

**Phase to address:** Theme Phase

---

### Pitfall 60: `.raw` Import Replaces Entire Program vs Appending

**What goes wrong:**
HP-41 hardware behavior: importing a program from a card reader REPLACES the program memory partition bounded by the target global label (or inserts a new partition). The current `state.program: Vec<Op>` is a flat list — replacing one "program" within it means finding the LBL/END boundaries for that program.

If import is implemented as `state.program = decoded_ops`, the user loses all existing programs. If import is implemented as "append to existing program memory", the user accumulates programs without any way to replace a specific one. Neither is fully correct.

**How to avoid:**
- Define the import semantics explicitly before implementation: recommended approach is "replace existing program with the same LBL name, or append if no matching LBL exists."
- The existing card reader implementation (`Op::Rprgm`) provides a reference: look at how `cardreader::cards.rs` handles program insertion.
- Add an import mode enum to the UI: `Replace All`, `Replace Matching Label`, `Append`.

**Warning signs:**
- Import wipes all other programs in memory.
- Import of a program with `LBL "QUAD"` when one already exists creates duplicate labels.

**Phase to address:** `.raw` Import/Export Phase

---

### Pitfall 61: X-MEM `EMROOM` Calculation Depends on Main Memory SIZE

**What goes wrong:**
On the HP-41CX, EMROOM (available X-MEM registers) is not simply `total_xmem_capacity - used_registers`. Available X-MEM space depends on how much main memory is allocated to data registers (set by SIZE), because X-MEM and main memory share the 1024-register address space at the hardware level.

An emulator that treats X-MEM capacity as a fixed constant (e.g., always 600 registers free) will report incorrect EMROOM values when the user has a large SIZE setting.

**How to avoid:**
- Implement EMROOM as: `XMEM_TOTAL_CAPACITY - used_xmem_bytes - overflow_from_main_memory(state.regs.len())`.
- Consult the HP-41CX Owner's Manual EMROOM section for the exact formula.
- Add a unit test: `SIZE 64` → EMROOM should be less than default.

**Note:** This is a behavioral fidelity issue. For MVP purposes, using a fixed capacity with a clear comment about the approximation is acceptable. Document as a divergence if shipped with the simplification.

**Warning signs:**
- EMROOM returns the same value regardless of how many data registers are allocated.
- Programs that check EMROOM before storing a file report "enough room" when there isn't.

**Phase to address:** X-MEM Core Phase

---

### Pitfall 62: GUI `resolveKeyId` Silently Returns `null` for Unmapped Keys Instead of Falling Through to XEQ-by-Name

**What goes wrong:**
CLI behavior: an unknown key falls through to `xeq_by_name_local_resolve` (the XEQ-by-name modal). A power user can type `F` in CLI to open the XEQ modal. In the GUI `resolveKeyId`, unmapped keys return `null` and are silently ignored (line 612: `if (keyId === null) return;`).

New keyboard shortcuts added during parity work may be wired correctly in `resolveKeyId` but resolve to a string ID that `key_map.rs::resolve` doesn't handle. The result is a toast error from the Rust backend saying "unknown key" — which is correct behavior per D-25.6 (never silently swallow). But the error may be confusing if the developer thought the key was wired.

**How to avoid:**
- Add `console.debug` logging in the `resolveKeyId` catch-all `?? null` branch so development builds surface unmapped physical keys.
- After GUI keyboard parity work, run a systematic test: press every key from CLI `key_to_op`'s reference table in the GUI and verify either correct dispatch or a clear XEQ-by-name modal.

**Warning signs:**
- Physical key press in GUI produces no response (silent ignore) when CLI would either dispatch an op or open a modal.
- Inconsistency between CLI and GUI for the same physical key on the same calculator function.

**Phase to address:** GUI Keyboard Parity Phase

---

## Technical Debt Patterns

| Shortcut | Immediate Benefit | Long-term Cost | When Acceptable |
|----------|-------------------|----------------|-----------------|
| Hardcode only "dark" theme in first pass | Faster delivery | Requires CSS architecture refactor when other themes added | Never — build the CSS variable architecture first even if only dark theme is implemented |
| Store theme in `CalcState` for simplicity | One fewer file location to manage | Conflates UI preference with calculator state; breaks CLI; adds serde complexity | Never |
| Implement `.raw` export for only the ~25 already-supported ops | Fast initial delivery | Users can't export any useful v3.x program | Acceptable for MVP with explicit error message listing unsupported ops |
| Treat X-MEM as `state.regs` extension | Reuses existing register infrastructure | `CLREG` corrupts X-MEM; address space conflicts | Never |
| Multi-program `.raw` import loads only first program silently | Simpler code | User data loss without error | Never — either load all or reject with clear message |
| Skip `#[serde(default)]` on transient X-MEM fields | One less annotation | Any v3.x save file fails to load after upgrade | Never |

---

## Integration Gotchas

| Integration | Common Mistake | Correct Approach |
|-------------|----------------|------------------|
| Theme + SVG gradients | Put colors in CSS custom properties on `:root`, expect SVG `<stop stopColor>` to pick them up | Pass theme config as React props to `<Keyboard>`; compute `stopColor` from theme at render time |
| Theme + CSS animation | Add new theme stylesheet that inadvertently resets `transform-box` | Keep animation invariants in a separate CSS layer that themes cannot override |
| `.raw` import + file dialog | Use Tauri `dialog::FileDialogBuilder` without `add_filter(".raw")` | Always add MIME/extension filter so users don't accidentally try to import autosave.json |
| `.raw` export + unsupported ops | Return `CardData` error that bubbles to a generic toast | Provide specific error listing which ops cannot be encoded; suggest HP41UC as external tool |
| X-MEM + `migrate_after_load()` | Add X-MEM fields but forget to initialize them in migrate | Add explicit X-MEM initialization arm in `migrate_after_load()` with version-gated logic |
| Onboarding + shared autosave | Check for onboarding-seen flag in `CalcState` | Use OS config dir (separate from autosave path) via `dirs::config_dir()` |
| GUI keyboard parity + `resolveKeyId` | Audit only `KEY_DEFS` on-screen path, miss physical keyboard | Systematically diff `hp41-cli/src/keys.rs::key_to_op` against `App.tsx::resolveKeyId` MAP |
| X-MEM + 4-way exhaustive match | Add `Op::EmDir` / `Op::EmRoom` / `Op::EmReg` variants and forget one of the four match locations | Follow existing XROM module pattern: all 4 match sites in the same commit |
| `.raw` + multi-program concatenation | Reject with cryptic "trailing bytes" error | Surface a human-readable error explaining multi-program files and offering alternatives |

---

## Performance Traps

| Trap | Symptoms | Prevention | When It Breaks |
|------|----------|------------|----------------|
| Onboarding JSON data loaded at startup | Cold-start regression (currently 2.2ms) | Use `OnceLock` or lazy initialization same as `help_data.rs` pattern | Immediately visible if onboarding data is large (>10KB) |
| Theme switching causes full React re-render of `<Keyboard>` | Key click latency spike after theme change | Memoize `<Keyboard>` with `React.memo`; only re-render when theme prop changes | Each theme switch; not a hot path but noticeable |
| X-MEM `xmem_files` serialized in autosave when no X-MEM is used | autosave.json grows silently | Use `#[serde(skip_serializing_if = "Vec::is_empty")]` on `xmem_files` | Once user runs `NEWM` and creates first X-MEM file |

---

## UX Pitfalls

| Pitfall | User Impact | Better Approach |
|---------|-------------|-----------------|
| Theme selector in a buried settings panel | Power users can't find it | Add theme selector to the help overlay (`?`) or a visible top-bar button |
| `.raw` import replaces all programs without warning | User loses programs they worked on | Show a confirmation dialog: "Import will replace program memory. Continue?" |
| Onboarding can't be re-triggered | New users who dismissed accidentally can't recover | Always-available "Show Quick Start" in help overlay or `?` overlay header |
| X-MEM EMDIR listing is CLI-only | GUI users have no way to see X-MEM contents | Add EMDIR output to the program listing panel or a separate X-MEM panel |
| `.raw` export silently fails for XROM ops | User doesn't know why the file is empty/missing | Show a list of unsupported ops in the error; offer to export the ops that are supported |

---

## "Looks Done But Isn't" Checklist

- [ ] **Theme system:** Verify `transform-box: fill-box` survives in all theme variants with a computed style test.
- [ ] **Theme system:** Verify SVG gradient stops change color (not just text/background) when theme is switched.
- [ ] **Theme system:** Verify theme preference is persisted to OS config dir, not `autosave.json`.
- [ ] **`.raw` import:** Verify that multi-program files produce a clear error message, not a silent partial load.
- [ ] **`.raw` export:** Verify that exporting a program containing XROM calls produces a clear error message listing which ops could not be encoded.
- [ ] **`.raw` round-trip:** Verify `encode_program(decode_program(bytes)) == bytes` for a representative set of community `.raw` files.
- [ ] **X-MEM isolation:** Verify `CLREG` does NOT clear X-MEM file data.
- [ ] **X-MEM serde:** Verify loading a v3.3 `autosave.json` fixture succeeds after adding X-MEM fields.
- [ ] **X-MEM 4-way match:** Verify all new X-MEM `Op` variants appear in `dispatch`, `execute_op`, CLI `op_display_name`, GUI `op_display_name`.
- [ ] **GUI keyboard parity:** Verify systematic diff of `key_to_op` vs `resolveKeyId` shows zero gaps.
- [ ] **Onboarding:** Verify deleting `autosave.json` causes onboarding to re-trigger.
- [ ] **Onboarding:** Verify onboarding does NOT re-trigger when only the calculator state is reset (SIZE, CLREG, etc.).

---

## Recovery Strategies

| Pitfall | Recovery Cost | Recovery Steps |
|---------|---------------|----------------|
| Theme breaks SVG animation (P51) | LOW | Add `transform-box: fill-box` back to the problematic theme CSS; add regression test |
| Multi-program `.raw` corrupt import (P52) | MEDIUM | Revert import; implement `decode_all_programs()` or improve error message |
| X-MEM fields missing serde(default) break saves (P53) | HIGH | Must add `#[serde(default)]` AND increment save format version; existing saves may not be recoverable without migration code |
| `resolveKeyId` gaps discovered post-ship (P54) | LOW | Add missing key to `resolveKeyId` MAP; no core change required |
| SVG gradient colors not themed (P55) | MEDIUM | Refactor `<Keyboard>` to accept theme props; replace hardcoded JSX color literals |
| X-MEM shares address space with `state.regs` (P56) | HIGH | Major refactor; all X-MEM ops must be rewritten to use isolated field |
| Onboarding in wrong location (P57) | LOW | Move flag from `CalcState` to OS config dir; clear existing entries in migrate |

---

## Pitfall-to-Phase Mapping

| Pitfall | Prevention Phase | Verification |
|---------|------------------|--------------|
| P51: SVG animation invariant | Theme Phase | Vitest computed style test for `transform-box` on key elements |
| P52: Multi-program `.raw` rejection | `.raw` I/O Phase | Test with a known multi-program community `.raw` file; verify error message is human-readable |
| P53: X-MEM serde backward compat | X-MEM Core Phase | CI test loading v3.3 fixture JSON; `just` target that deserializes a pinned save file |
| P54: `resolveKeyId` gaps | GUI Keyboard Parity Phase | Systematic diff test: `key_to_op` entries vs `resolveKeyId` entries |
| P55: SVG gradient CSS variables | Theme Phase | Manual verify all gradient surfaces change color on theme switch |
| P56: X-MEM register isolation | X-MEM Core Phase | Test: `CLREG` after `NEWM` / `EMREG` store; verify X-MEM data survives |
| P57: Onboarding in wrong location | Onboarding Phase | Test: delete autosave.json; verify onboarding re-triggers; delete OS config; verify same |
| P58: XROM encoding missing | `.raw` I/O Phase | Test exporting a program with `XEQ "QUAD"`; verify error message lists unsupported ops |
| P59: Theme in CalcState | Theme Phase | Code review gate: `CalcState` diff must not contain `theme` field |
| P60: Import replaces vs appends | `.raw` I/O Phase | Test: import with existing programs; verify expected behavior (not silent wipe) |
| P61: EMROOM depends on SIZE | X-MEM Core Phase | Test: `SIZE 64` then `EMROOM`; compare result vs `SIZE 0` |
| P62: Silent `null` from `resolveKeyId` | GUI Keyboard Parity Phase | Debug logging in `null` catch-all; systematic key-by-key test against CLI reference |

---

## Sources

- `hp41-core/src/cardreader/raw.rs` — existing `.raw` codec; current encoding subset, END marker format, `0xCF`/`0xCD` disambiguation (HIGH confidence — codebase)
- `hp41-core/src/state.rs` — `CalcState` serde patterns, `#[serde(default)]` / `#[serde(skip)]` discipline, `rand_seed` and `adv_tvm_state` as the two `default`-without-`skip` precedents (HIGH confidence — codebase)
- `hp41-gui/src/App.css` — `.key { transform-box: fill-box; }` invariant with inline documentation (HIGH confidence — codebase)
- `hp41-gui/src/Keyboard.tsx` — hardcoded JSX `stopColor` props in `<defs>`, `getKeyGrad` function (HIGH confidence — codebase)
- `hp41-gui/src/App.tsx` lines 108–161 — `resolveKeyId` handwritten MAP; gap with `key_to_op` in `hp41-cli/src/keys.rs` (HIGH confidence — codebase)
- `CLAUDE.md` Frozen Invariants section — 4-way exhaustive match, serde backward compat rules, `#[serde(default)]` discipline, save-file compat (HIGH confidence — project constraints)
- [Using HP-41C RAW/P41 files with HP-42X](https://www.hrastprogrammer.com/hp42x/rawfiles.htm) — RAW file has no header, multi-program structure, NULL byte between consecutive numerics (MEDIUM confidence — community doc)
- [HP-41 Tagged RAW Files discussion](https://forum.hp41.org/viewtopic.php?f=21&t=610) — identification ambiguity, emulator-specific END metadata (MEDIUM confidence — community forum)
- [HP-41 FOCAL byte encoding search results](https://id-phy.orgfree.com/HP41/HP41_ProgEnv.html) — XROM two-byte encoding `0xAx`, GTO/XEQ/LBL alpha encoding `1D/1E/CF Fx ...` (MEDIUM confidence — community reference)
- [SVG light/dark mode CSS pitfalls — web.dev theming](https://web.dev/learn/design/theming) — CSS custom properties do not reach SVG attribute values, `currentColor` workaround (HIGH confidence — web platform docs)
- [HP-41 Extended Memory overview](https://archived.hpcalc.org/museumforum/thread-54029.html) — X-MEM register count, address space sharing with main memory, EMDIR/EMROOM behavioral description (MEDIUM confidence — community forum)

---
*Pitfalls research for: HP-41 Calculator Emulator v4.0 Platform Maturity*
*Researched: 2026-05-27*
