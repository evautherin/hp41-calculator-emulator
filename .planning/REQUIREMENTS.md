# Requirements: HP-41 Calculator Emulator v4.0

**Defined:** 2026-05-27
**Core Value:** Faithful HP-41 RPN fidelity — the four-level stack, stack-lift semantics, display, and keystroke programming must behave identically to the original hardware; everything else is secondary.

## v4.0 Requirements

Requirements for v4.0 Platform Maturity. Each maps to roadmap phases.

### GUI Infrastructure

- [ ] **INFRA-01**: `prefs.rs` Tauri backend for persistent user preferences (separate from CalcState), stored in `~/.hp41/prefs.json`
- [ ] **INFRA-02**: `get_prefs` / `set_pref` Tauri commands with Tauri v2.11 permission TOMLs

### Skin Themes

- [ ] **THEME-01**: User can select from 4 built-in skin themes (dark, light, classic beige, high-contrast)
- [ ] **THEME-02**: Selected theme persists across app restarts via `~/.hp41/prefs.json`
- [ ] **THEME-03**: SVG key press animations remain functional in all themes (`transform-box: fill-box` preserved)
- [ ] **THEME-04**: High-contrast theme meets WCAG AA contrast ratios
- [ ] **THEME-05**: Theme preference stored separately from CalcState (never in `autosave.json`)

### Onboarding & Reference

- [ ] **ONBOARD-01**: First-run quick-start overlay introduces RPN basics, key layout, and how to access functions
- [ ] **ONBOARD-02**: Quick-start can be re-opened from help menu or `?` overlay
- [ ] **ONBOARD-03**: Searchable in-app function reference with examples and usage notes
- [ ] **ONBOARD-04**: Function reference covers all 5 XROM modules + ~130 built-in functions (~350 entries)
- [ ] **ONBOARD-05**: "Seen" flag stored in `prefs.json`, not CalcState

### GUI Keyboard Parity

- [ ] **KBD-01**: Card reader shortcuts (Ctrl+W/R/D/F) work in GUI physical keyboard
- [ ] **KBD-02**: F5 triggers manual save in GUI
- [ ] **KBD-03**: Physical keyboard shortcut reference displayed in `?` overlay
- [ ] **KBD-04**: All CLI key bindings have equivalent GUI keyboard paths (audit-verified)

### `.raw` File I/O

- [ ] **RAW-01**: User can import a single-program `.raw` file into calculator memory
- [ ] **RAW-02**: User can export a program to `.raw` format
- [ ] **RAW-03**: Multi-program `.raw` archive files handled (import all or clear error)
- [ ] **RAW-04**: Import/export uses native OS file dialog via `tauri-plugin-dialog`
- [ ] **RAW-05**: XROM instructions in imported `.raw` files preserved correctly
- [ ] **RAW-06**: CLI equivalent via `--import-raw` / `--export-raw` flags

### Extended Memory (X-MEM)

- [x] **XMEM-01**: EMDIR lists all files in extended memory with names, types, and sizes
- [x] **XMEM-02**: EMROOM reports available register space in extended memory
- [x] **XMEM-03**: SAVEP saves a named program to extended memory
- [x] **XMEM-04**: GETP retrieves a named program from extended memory
- [x] **XMEM-05**: SAVED saves data registers to a named file in extended memory
- [x] **XMEM-06**: GETD retrieves data registers from a named file in extended memory
- [x] **XMEM-07**: EMREG accesses register N within the active X-MEM file
- [ ] **XMEM-08**: X-MEM state persists across save/load with `#[serde(default)]` backward compat
- [ ] **XMEM-09**: X-MEM storage isolated from main registers and `adv_matrices` (D-43.5 pattern)
- [ ] **XMEM-10**: CLI + GUI integration (4-way exhaustive match, `op_display_name`, help overlay)

## v4.1+ Requirements

Deferred to future release. Tracked but not in current roadmap.

### Extended Memory Extensions

- **XMEM-F01**: X-MEM ASCII file type support
- **XMEM-F02**: X-MEM STATUS file type support

### File Exchange Extensions

- **RAW-F01**: LIF disk image mounting

### Onboarding Extensions

- **ONBOARD-F01**: Interactive keystroke tutorial (step-by-step guided exercises)

## Out of Scope

| Feature | Reason |
|---------|--------|
| External `.gif` bitmap skins (Free42 style) | SVG keyboard not suited for bitmap overlays; built-in presets sufficient |
| Cycle-accurate Nut CPU simulation | High effort, low user value vs. behavioral emulation (permanent exclusion) |
| HP-copyrighted ROM image redistribution | Legal risk (permanent exclusion) |
| Mobile (iOS/Android) | Separate major initiative, not v4.0 scope |
| Cloud sync | Privacy and infrastructure cost |
| Interrupting control alarm execution | Requires re-entrancy against 4-level call stack (deferred from v3.2) |

## Traceability

Which phases cover which requirements. Updated during roadmap creation.

| Requirement | Phase | Status |
|-------------|-------|--------|
| INFRA-01 | Phase 48 | Pending |
| INFRA-02 | Phase 48 | Pending |
| THEME-01 | Phase 48 | Pending |
| THEME-02 | Phase 48 | Pending |
| THEME-03 | Phase 48 | Pending |
| THEME-04 | Phase 48 | Pending |
| THEME-05 | Phase 48 | Pending |
| ONBOARD-01 | Phase 49 | Pending |
| ONBOARD-02 | Phase 49 | Pending |
| ONBOARD-03 | Phase 49 | Pending |
| ONBOARD-04 | Phase 49 | Pending |
| ONBOARD-05 | Phase 49 | Pending |
| KBD-01 | Phase 49 | Pending |
| KBD-02 | Phase 49 | Pending |
| KBD-03 | Phase 49 | Pending |
| KBD-04 | Phase 49 | Pending |
| RAW-01 | Phase 50 | Pending |
| RAW-02 | Phase 50 | Pending |
| RAW-03 | Phase 50 | Pending |
| RAW-04 | Phase 50 | Pending |
| RAW-05 | Phase 50 | Pending |
| RAW-06 | Phase 50 | Pending |
| XMEM-01 | Phase 51 | Complete |
| XMEM-02 | Phase 51 | Complete |
| XMEM-03 | Phase 51 | Complete |
| XMEM-04 | Phase 51 | Complete |
| XMEM-05 | Phase 51 | Complete |
| XMEM-06 | Phase 51 | Complete |
| XMEM-07 | Phase 51 | Complete |
| XMEM-08 | Phase 52 | Pending |
| XMEM-09 | Phase 52 | Pending |
| XMEM-10 | Phase 52 | Pending |

**Coverage:**
- v4.0 requirements: 32 total
- Mapped to phases: 32
- Unmapped: 0 ✓

---
*Requirements defined: 2026-05-27*
*Last updated: 2026-05-27 — traceability filled during roadmap creation (all 32 requirements mapped)*
