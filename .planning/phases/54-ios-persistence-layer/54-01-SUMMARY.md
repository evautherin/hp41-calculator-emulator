---
phase: 54-ios-persistence-layer
plan: "01"
subsystem: hp41-gui/persistence
tags: [ios, persistence, tauri, path-resolution, backward-compat]
dependency_graph:
  requires: []
  provides: [state_path_for_app, prefs_path_for_app, v40-fixture]
  affects: [hp41-gui/src-tauri/src/persistence.rs, hp41-gui/src-tauri/src/prefs.rs, hp41-gui/src-tauri/src/lib.rs, hp41-gui/src-tauri/src/commands.rs]
tech_stack:
  added: []
  patterns: [cfg(mobile)/cfg(not(mobile)) platform gate, AppHandle-aware path resolver, StateFile backward-compat fixture]
key_files:
  created:
    - hp41-gui/src-tauri/tests/fixtures/v40-autosave.json
  modified:
    - hp41-gui/src-tauri/src/persistence.rs
    - hp41-gui/src-tauri/src/prefs.rs
    - hp41-gui/src-tauri/src/lib.rs
    - hp41-gui/src-tauri/src/commands.rs
decisions:
  - "state_path_for_app uses #[cfg(mobile)] / #[cfg(not(mobile))] — not #[cfg(target_os=ios)] — to include aarch64-apple-ios-sim (RESEARCH anti-patterns)"
  - "Fallback chain in mobile branch: app_local_data_dir() then dirs::home_dir()+Library+Application Support+ch.talent-factory.hp41 (Pitfall 3: NOT .hp41)"
  - "#[allow(unused_variables)] on both resolver functions since handle is intentionally unused in #[cfg(not(mobile))] branch on desktop"
  - "Fixture generated programmatically via save_state() then committed — matches existing pattern in hp41-core/tests/fixtures"
  - "include_str! path from src/persistence.rs to tests/fixtures/ is ../tests/fixtures/v40-autosave.json (one level up from src/)"
metrics:
  duration: "474s (~8 min)"
  completed: "2026-06-02T18:17:15Z"
  tasks_completed: 3
  files_changed: 5
requirements: [PERSIST-01, PERSIST-03]
---

# Phase 54 Plan 01: iOS Persistence Layer — Path Resolver + Backward-Compat Fixture Summary

AppHandle-aware `state_path_for_app` / `prefs_path_for_app` resolvers added; all five hard-coded `default_*_path()` call sites in `lib.rs` and `commands.rs` atomically replaced; v4.0-era `StateFile`-wrapped fixture committed with backward-compat test green.

## What Was Built

### Task 1 — AppHandle-aware path resolvers

Added `pub fn state_path_for_app(handle: &tauri::AppHandle) -> PathBuf` to `persistence.rs`
and `pub fn prefs_path_for_app(handle: &tauri::AppHandle) -> PathBuf` to `prefs.rs`.

Both functions use the `#[cfg(mobile)]` / `#[cfg(not(mobile))]` idiom (matching the
existing `#[cfg(desktop)]` autostart gate in `lib.rs:46-50`):

- **Mobile branch:** calls `handle.path().app_local_data_dir()` first; on `Err` (Tauri
  #12552 fallback) logs via `eprintln!` and falls back to
  `dirs::home_dir() → Library → Application Support → ch.talent-factory.hp41`.
  This is the correct iOS sandbox Application Support path (D-54.1).
- **Desktop branch:** delegates unchanged to `default_state_path()` / `default_prefs_path()`.

Both `default_state_path()` and `default_prefs_path()` are byte-for-byte unchanged.
`cards.rs` is untouched (D-54.3a).

New host unit tests added:
- `test_default_state_path_ends_with_dot_hp41_autosave` — desktop path unchanged guard
- `test_mobile_fallback_path_construction` — verifies fallback contains "Application Support"
  and bundle id "ch.talent-factory.hp41", and does NOT land at the Pitfall 3 container-root
  `.hp41/` location

### Task 2 — Rewire all five call sites

All five call sites atomically updated (RESEARCH Pitfall 1 — partial migration silently
breaks PERSIST-02 on iOS):

| # | File | Old | New |
|---|------|-----|-----|
| 1 | `lib.rs:64` | `prefs::default_prefs_path()` | `prefs::prefs_path_for_app(app.handle())` |
| 2 | `lib.rs:72` | `persistence::default_state_path()` | `persistence::state_path_for_app(app.handle())` |
| 3 | `lib.rs:104` | `default_state_path()` inside thread | `state_path_for_app(&handle)` resolved BEFORE `thread::spawn`, move-captured PathBuf |
| 4 | `commands.rs save_state` | `default_state_path()` | `state_path_for_app(&app)` + new `app: AppHandle` param |
| 5 | `commands.rs set_pref` | `default_prefs_path()` | `prefs_path_for_app(&app)` + new `app: AppHandle` param |

Auto-save thread explicitly keeps the 30s sleep (D-54.2a). CR-01 (clone-under-lock then
release before disk I/O) is preserved unchanged. Tauri inject for `app: AppHandle` requires
no capability/permission change.

### Task 3 — v4.0 backward-compat fixture + test + P-iOS-29 audit

**Fixture:** `hp41-gui/src-tauri/tests/fixtures/v40-autosave.json` — StateFile-wrapped
(`{"version":1,"state":{…}}`), programmatically generated via `save_state()`. Contains:
- `xrom_modules: 31` (0b0001_1111, all 5 XROM modules)
- `rand_seed: "3.141592650"` (non-zero — first serde-exception field)
- `adv_tvm_state: {n:12, i:8, pv:10000, …}` (non-null — second serde-exception field)
- `xmem_files: [{name:"DATFILE", kind:"Data", data:[], reg_count:2}]` (X-MEM state)
- `xmem_active_file: "DATFILE"`

**Test:** `test_loads_v40_autosave_fixture` in `persistence.rs` `#[cfg(test)]` module;
uses `include_str!("../tests/fixtures/v40-autosave.json")`. Asserts:
- `!loaded.is_running` (Pitfall 4 / D-54.2b)
- `loaded.xrom_modules == 0b0001_1111u8`
- `!loaded.xmem_files.is_empty()`
- `loaded.xmem_active_file == Some("DATFILE")`
- `loaded.rand_seed != HpNum::zero()` (serde-exception policy)
- `loaded.adv_tvm_state.is_some()` (serde-exception policy)

## P-iOS-29 Absolute-Path Audit Conclusion

Audit scope: all fields on `CalcState` in `hp41-core/src/state.rs`.

**Finding: PASS — No `CalcState` field persists an absolute filesystem path.**

Verified fields:
- `xmem_active_file: Option<String>` — stores the X-MEM file **name** (e.g. `"DATFILE"`), not a path. Confirmed in `hp41-core/src/ops/xmem/mod.rs:30`.
- `XmemFile.name: String` — file identifier only, no path component. Confirmed.
- `key_assignments: HashMap<char, String>` — maps key characters to function-name strings. No path strings.
- `alpha_reg: String` — ALPHA register display content. No path strings.
- `adv_tvm_state` / all `HpNum` fields — numeric data only (rust_decimal serde::str).
- No `PathBuf` or `std::path::Path` typed fields anywhere in `CalcState`. Confirmed by grep of `hp41-core/src/state.rs`.

**T-54-04 (path traversal via persisted path field) is mitigated.** Container-UUID changes across iOS reinstalls cannot break state because no absolute path is stored.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] `include_str!` path was one level too deep**
- **Found during:** Task 3 compile
- **Issue:** PATTERNS.md suggested `../../tests/fixtures/…` but `src/persistence.rs` is already inside `src/`, so the correct relative path is `../tests/fixtures/…`
- **Fix:** Changed `include_str!("../../tests/fixtures/v40-autosave.json")` → `include_str!("../tests/fixtures/v40-autosave.json")`
- **Files modified:** `hp41-gui/src-tauri/src/persistence.rs`
- **Commit:** 7548866

**2. [Rule 2 - Missing functionality] Suppress unused-variable warning on AppHandle resolver functions**
- **Found during:** Task 2 build
- **Issue:** On desktop, `handle` parameter is unused in `#[cfg(not(mobile))]` branch (intentional — it IS used in the `#[cfg(mobile)]` branch). Rust emits `-W unused-variables` warning.
- **Fix:** Added `#[allow(unused_variables)]` attribute with explanatory comment on both `state_path_for_app` and `prefs_path_for_app` functions.
- **Files modified:** `hp41-gui/src-tauri/src/persistence.rs`, `hp41-gui/src-tauri/src/prefs.rs`
- **Commit:** 054bac3

**3. [Rule 1 - Bug] HpNum::from_str not available — use Decimal::from_str**
- **Found during:** Task 3 fixture generation test compilation
- **Issue:** `HpNum` does not implement `std::str::FromStr`; construction is via `HpNum::from(Decimal)`.
- **Fix:** Changed generator to use `HpNum::from(Decimal::from_str("3.14159265").unwrap())`.
- **Files modified:** Fixture generator test (intermediate; not in final commit)

## Known Stubs

None. All resolver functions contain complete implementation. Fixture file is fully
populated with v4.0 fields. No placeholder values.

## Threat Flags

None. This plan only changes path resolution dispatch — no new network endpoints, no new
auth paths, no new schema changes at trust boundaries. T-54-01 through T-54-05 are all
addressed as documented in the plan's threat model.

## Self-Check: PASSED

Files verified to exist:
- `hp41-gui/src-tauri/src/persistence.rs` — contains `fn state_path_for_app` ✓
- `hp41-gui/src-tauri/src/prefs.rs` — contains `fn prefs_path_for_app` ✓
- `hp41-gui/src-tauri/tests/fixtures/v40-autosave.json` — contains `"version": 1` ✓

Commits verified:
- `4502ea6` feat(54-01): add AppHandle-aware path resolvers ✓
- `054bac3` feat(54-01): rewire all five path call sites ✓
- `7548866` test(54-01): add v4.0 backward-compat fixture + test ✓
