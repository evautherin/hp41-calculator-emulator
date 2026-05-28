---
gsd_state_version: 1.0
milestone: v4.0
milestone_name: Platform Maturity
status: executing
last_updated: "2026-05-28T05:03:44.522Z"
last_activity: 2026-05-28 -- Phase 51 planning complete
progress:
  total_phases: 5
  completed_phases: 3
  total_plans: 13
  completed_plans: 11
  percent: 60
---

# Project State: HP-41 Calculator Emulator

---

## Project Reference

See: .planning/PROJECT.md (updated 2026-05-27 after v3.3 shipped)

**Core value:** Faithful HP-41 RPN fidelity — four-level stack, stack-lift semantics, display, and keystroke programming must behave identically to original hardware; everything else is secondary.

**Current focus:** Phase 51 — x-mem-core (context gathered, ready to plan)

---

## Current Position

Phase: 51 (x-mem-core) — CONTEXT GATHERED
Plan: not yet planned
Status: Ready to execute
Last activity: 2026-05-28 -- Phase 51 planning complete

## Progress Bar

```
v4.0 Platform Maturity
Phase 48 ██████████ 100%  Phase 49 ██████████ 100%  Phase 50 ██████████ 100%  Overall ██████░░░░ 60%
```

| Phase | Goal | Status |
|-------|------|--------|
| 48 | GUI Infrastructure + Theming | Complete |
| 49 | Onboarding + GUI Keyboard Parity | Complete |
| 50 | .raw File I/O | Complete |
| 51 | X-MEM Core | Context gathered |
| 52 | Test Hardening + Documentation | Not started |

## Performance Metrics (v3.3 ship baseline)

| Metric | Target | Last measured (v3.3) |
|--------|--------|----------------------|
| Cold-start latency | <= 0.5 s | 2.2 ms (M1) |
| Key-press latency | <= 50 ms | ~65 ns/op |
| `hp41-core` line coverage | >= 95 % | ~93 % (denominator dilution from ~32K new LOC) |
| `hp41-core` region coverage | >= 93 % | ~95 % |
| Numerical accuracy | >= 98 % | 98.86 % (791+30+22 = 843 cases) |
| Panics in `hp41-core` | 0 | 0 |
| Free42 contamination | 0 | 0 (18-token guard) |
| CI platforms | Win/macOS/Ubuntu | All green |
| Tests passing | — | 3371 (up from 3262 at v3.3) |

---

## Accumulated Context

### Decisions (pre-resolved from research)

- **Preferences backend:** `prefs.rs` in `hp41-gui/src-tauri/src/` — hand-coded `GuiPrefs { theme, onboarding_done }` via `serde_json`, stored in `~/.hp41/prefs.json` (separate from `autosave.json`). `tauri-plugin-store` rejected (overkill for 2 fields).
- **Theme implementation:** CSS custom properties + `data-theme` attribute on `<body>` — zero libraries. 4 presets: dark, light, classic-beige, high-contrast.
- **SVG animation safety (P51):** Every `[data-theme]` CSS block must preserve `transform-box: fill-box` + `transform-origin: center` on `.key`. Verified per pitfall.
- **SVG gradient stops (P55):** `<defs>` gradient stops do not inherit CSS variables — pass theme config as React props to `<Keyboard>`.
- **.raw codec:** Already fully implemented in `hp41-core/src/cardreader/raw.rs`. Phase 50 is a frontend integration task — wire `encode_program`/`decode_program` to Tauri commands + CLI flags. New dep: `tauri-plugin-dialog` 2.4.2 (Rust) / 2.7.1 (npm).
- **X-MEM storage isolation (P56):** `xmem_files: Vec<XmemFile>` on `CalcState` with `#[serde(default)]` — NEVER touches `state.regs` or `adv_matrices` (D-43.5 pattern repeated).
- **X-MEM + .raw dependency:** SAVEP/GETP delegate to the `.raw` codec; Phase 51 depends on Phase 50.
- **Zero new runtime deps:** Policy from v3.0 continues. `tauri-plugin-dialog` is a Tauri plugin (frontend dep), not a new `hp41-core` runtime dep — policy unbroken.

### Pitfalls to watch

| ID | Description |
|----|-------------|
| P51 | SVG animation broken by theme CSS — preserve `transform-box: fill-box` in every theme block |
| P52 | `.raw` multi-program files silently rejected — implement `decode_all_programs()` or clear error |
| P53 | X-MEM missing `#[serde(default)]` breaks saves — CI fixture test with pinned v3.3 save file |
| P55 | SVG `<defs>` gradient stops ignore CSS vars — pass theme config as React props to `<Keyboard>` |
| P56 | X-MEM shares address space with `state.regs` — use dedicated `xmem_files: Vec<XmemFile>` |
| P59 | Theme/onboarding flag in CalcState — must live in `~/.hp41/prefs.json`, not `autosave.json` |

### Blockers

None.

### Pending Todos

- Run `/gsd-plan-phase 51` to plan Phase 51: X-MEM Core (CONTEXT.md ready)

---

## Deferred Items

From v3.3 milestone close — all confirmed complete:

| Category | Item | Status |
|----------|------|--------|
| Deferred | Interrupting control alarm execution | Still deferred; data model ready (D-38.4); requires call-stack re-entrancy |
| Deferred | Signed binary releases (cargo-dist + tauri-action) | Deferred post-v4.0 |

---

*State initialized: 2026-05-06*
*Last updated: 2026-05-28 — Phase 50 complete (.raw File I/O); Phase 51 (X-MEM Core) context gathered, ready to plan*
