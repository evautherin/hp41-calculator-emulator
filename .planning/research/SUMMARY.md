# Research Summary: v4.0 Platform Maturity

**Synthesized:** 2026-05-27
**Sources:** STACK.md, FEATURES.md, ARCHITECTURE.md, PITFALLS.md
**Confidence:** HIGH (Phases 1-3) / MEDIUM (Phase 4 — X-MEM spec gap)

## Executive Summary

v4.0 adds five orthogonal platform capabilities to a feature-complete HP-41 emulator. All five areas are additive — none require redesigning existing subsystems. The recommended approach: shared GUI preferences infrastructure first (theme + onboarding flag), then `.raw` file I/O (the codec already fully exists in `hp41-core/src/cardreader/raw.rs`), and finally the Extended Memory model as the one genuinely novel core feature.

**Key finding:** The `.raw` codec is already fully implemented. The v4.0 `.raw` feature is almost entirely a frontend integration task — wire existing `encode_program`/`decode_program` to a Tauri file dialog and CLI flags.

## Stack Additions

| Addition | Version | Why |
|----------|---------|-----|
| `@tauri-apps/plugin-dialog` | 2.7.1 (npm) / 2.4.2 (Rust) | Native file picker for `.raw` I/O; only new dep |
| CSS custom properties + `data-theme` | — | Theming with zero libraries |
| `prefs.rs` (hand-coded) | — | `GuiPrefs { theme, onboarding_done }` via `serde_json` |

**What NOT to add:** Tailwind (dormant in devDeps, not worth migrating 394 lines of CSS). React Joyride (SVG keyboard has no clean DOM anchors). `tauri-plugin-store` (overkill for 2 fields).

## Feature Table Stakes vs Differentiators

| Feature | Table Stakes | Differentiator |
|---------|-------------|----------------|
| Themes | Dark + light preset | Classic beige + high-contrast (WCAG AA) |
| Onboarding | — | First-run RPN guide (no HP-41 emulator has this) |
| `.raw` I/O | Import single-program files | Multi-program archives; XROM 2-byte encoding |
| Keyboard parity | Ctrl+W/R/D/F card reader, F5 save | Shortcut reference in `?` overlay |
| X-MEM | EMDIR/EMROOM, SAVEP/GETP, SAVED/GETD | Full file-type catalog |

## Architecture — New/Modified Components

1. `hp41-gui/src-tauri/src/prefs.rs` (NEW) — preferences load/save, separate from CalcState
2. `hp41-gui/src/App.css` (MODIFY) — ~22 CSS custom properties, 4 `[data-theme]` blocks
3. `hp41-gui/src/Keyboard.tsx` (MODIFY) — SVG `fill` → CSS variable refs; theme props for gradient stops
4. `hp41-core/src/ops/xmem.rs` (NEW) — `ExtendedMemory` + `XMemFile`; isolated from `state.regs`
5. `hp41-gui/src-tauri/src/commands.rs` (MODIFY) — `import_raw`, `export_raw`, `get_prefs`, `set_pref`

## Critical Pitfalls

| ID | Risk | Prevention |
|----|------|------------|
| P51 | SVG animation broken by theme CSS | Preserve `transform-box: fill-box` in every theme block |
| P52 | `.raw` multi-program files silently rejected | Implement `decode_all_programs()` or clear error |
| P53 | X-MEM missing `#[serde(default)]` breaks saves | CI fixture test with pinned v3.3 save file |
| P55 | SVG `<defs>` gradient stops ignore CSS vars | Pass theme config as React props to `<Keyboard>` |
| P56 | X-MEM shares address space with `state.regs` | Dedicated `xmem_files: Vec<XmemFile>` field |
| P59 | Theme/onboarding flag in CalcState | Must live in `~/.hp41/prefs.json`, not `autosave.json` |

## Suggested Phase Order

1. **GUI Infrastructure + Theming** — `prefs.rs`, CSS variables, theme presets; zero core changes
2. **Onboarding + GUI Keyboard Parity** — depends on Phase 1 prefs; purely frontend
3. **`.raw` File I/O** — wire existing codec to Tauri commands + CLI flags; one new dep
4. **X-MEM Core + CLI/GUI Integration** — novel core feature; depends on Phase 3 codec
5. **Test Hardening + Documentation** — round-trip tests, backward compat, ADRs

**Phase ordering rationale:**
- Phase 1 before 2: `prefs.rs` backend shared by theme and onboarding
- Phase 3 before 4: X-MEM SAVEP/GETP delegate to `.raw` codec
- Phases 1+2 independent of 3+4 — can parallel if desired

## Research Gaps

- **X-MEM byte-level format:** HP-41CX Extended Functions/Memory Module OM (CHM 102650266) not web-accessible; data model sound but internal nibble layout unverified
- **XROM nibble encoding for `.raw` export:** verify against HP-41 Synthetic QRG before Phase 3
- **`.raw` import semantics:** replace vs append vs replace-matching-label needs explicit decision

---
*Synthesized from STACK.md + FEATURES.md + ARCHITECTURE.md + PITFALLS.md on 2026-05-27*
