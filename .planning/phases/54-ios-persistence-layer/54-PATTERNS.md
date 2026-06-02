# Phase 54: iOS Persistence Layer - Pattern Map

**Mapped:** 2026-06-02
**Files analyzed:** 6 (5 modified Rust/React, 1 created fixture)
**Analogs found:** 6 / 6

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `hp41-gui/src-tauri/src/persistence.rs` | service | file-I/O | `hp41-gui/src-tauri/src/persistence.rs` (self — extend existing) | exact |
| `hp41-gui/src-tauri/src/prefs.rs` | service | file-I/O | `hp41-gui/src-tauri/src/persistence.rs` | exact (stated as mirror in file header) |
| `hp41-gui/src-tauri/src/lib.rs` | config/wiring | request-response | `hp41-gui/src-tauri/src/lib.rs` lines 46–50 (`#[cfg(desktop)]` autostart gate) | exact |
| `hp41-gui/src-tauri/src/commands.rs` | controller | request-response | `hp41-gui/src-tauri/src/commands.rs` lines 619–623 (`import_raw_dialog` with `AppHandle`) | role-match |
| `hp41-gui/src/App.tsx` | component | event-driven | `hp41-gui/src/App.tsx` lines 933–936 (`keydown` useEffect) | exact |
| `hp41-gui/src-tauri/tests/fixtures/v40-autosave.json` | test fixture | — | `hp41-core/tests/fixtures/v33-autosave.json` (bare CalcState) + `persistence.rs` `StateFile` wrapper | partial — GUI fixture must add `{"version":1,"state":{…}}` wrapper |

---

## Pattern Assignments

### `hp41-gui/src-tauri/src/persistence.rs` (service, file-I/O)

**Analog:** Self — add `state_path_for_app()` alongside the existing `default_state_path()`.

**Existing `default_state_path()` pattern** (lines 32–37) — the desktop function to keep unchanged:
```rust
pub fn default_state_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".hp41")
        .join("autosave.json")
}
```

**`#[cfg(mobile)]` / `#[cfg(not(mobile))]` branching pattern** — sourced from `lib.rs` lines 46–50 (desktop-only plugin gate). New function follows the same idiom:
```rust
// lib.rs lines 46-50 — the established #[cfg(desktop)] gate this phase mirrors with #[cfg(mobile)]:
#[cfg(desktop)]
let builder = builder.plugin(tauri_plugin_autostart::init(
    tauri_plugin_autostart::MacosLauncher::LaunchAgent,
    None,
));
```

**New function to add** (after `default_state_path()`, before `save_state()`):
```rust
/// AppHandle-aware path resolver. On mobile (iOS) uses app_local_data_dir()
/// → Library/Application Support/<bundle_id>/autosave.json.
/// On desktop delegates to default_state_path() — desktop behavior unchanged.
pub fn state_path_for_app(handle: &tauri::AppHandle) -> PathBuf {
    #[cfg(mobile)]
    {
        handle
            .path()
            .app_local_data_dir()
            .unwrap_or_else(|e| {
                eprintln!("hp41: app_local_data_dir failed ({e}), falling back to HOME");
                dirs::home_dir()
                    .unwrap_or_else(|| PathBuf::from("."))
                    .join("Library")
                    .join("Application Support")
                    .join("ch.talent-factory.hp41")
            })
            .join("autosave.json")
    }
    #[cfg(not(mobile))]
    {
        default_state_path()
    }
}
```

**Existing backward-compat test pattern** (lines 157–184) — the v4.0 fixture test must mirror this exactly:
```rust
#[test]
fn test_loads_v1_format_state_file() {
    use hp41_core::num::HpNum;

    let path = temp_path("v1_compat");
    let fresh = CalcState::new();
    save_state(&path, &fresh).unwrap();

    // Strip v1.1-introduced fields from the serialized JSON to simulate a v1.0 save.
    let raw = fs::read_to_string(&path).unwrap();
    let mut value: serde_json::Value = serde_json::from_str(&raw).unwrap();
    let state_obj = value["state"].as_object_mut().expect("state must be a JSON object");
    for key in ["last_key_code", "reg_m", "reg_n", "reg_o"] {
        state_obj.remove(key);
    }
    fs::write(&path, value.to_string()).unwrap();

    let loaded = load_state(&path).expect("v1.0-format save must load via serde(default)");
    // assert loaded fields ...
    let _ = fs::remove_dir_all(path.parent().unwrap());
}
```

**`temp_path` helper** (lines 74–78) — reused verbatim in new tests:
```rust
fn temp_path(name: &str) -> PathBuf {
    std::env::temp_dir()
        .join(format!("hp41_test_{name}"))
        .join("state.json")
}
```

**Test module boilerplate** (lines 68–71):
```rust
#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use hp41_core::CalcState;
```

---

### `hp41-gui/src-tauri/src/prefs.rs` (service, file-I/O)

**Analog:** `hp41-gui/src-tauri/src/persistence.rs` — prefs.rs file header explicitly states "Design mirrors `persistence.rs`".

**Existing `default_prefs_path()` pattern** (lines 71–76) — keep unchanged:
```rust
pub fn default_prefs_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".hp41")
        .join("prefs.json")
}
```

**New function to add** — same `#[cfg(mobile)]` gate as `state_path_for_app()`, only the filename differs:
```rust
pub fn prefs_path_for_app(handle: &tauri::AppHandle) -> PathBuf {
    #[cfg(mobile)]
    {
        handle
            .path()
            .app_local_data_dir()
            .unwrap_or_else(|e| {
                eprintln!("hp41: app_local_data_dir failed ({e}), falling back to HOME");
                dirs::home_dir()
                    .unwrap_or_else(|| PathBuf::from("."))
                    .join("Library")
                    .join("Application Support")
                    .join("ch.talent-factory.hp41")
            })
            .join("prefs.json")
    }
    #[cfg(not(mobile))]
    {
        default_prefs_path()
    }
}
```

**Import to add** — `prefs.rs` currently has no `tauri` import; add alongside the existing `use std::path::{Path, PathBuf}` block:
```rust
// Existing imports (lines 16-19):
use std::fs;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
// Add:
// (tauri::AppHandle is used only in prefs_path_for_app; no other tauri dep in this module)
```

---

### `hp41-gui/src-tauri/src/lib.rs` (config/wiring, request-response)

**Analog:** Lines 46–50 (`#[cfg(desktop)]` autostart gate) and lines 64–116 (setup path + auto-save thread).

**Existing `#[cfg(desktop)]` gate pattern** (lines 46–50) — the exact idiom to copy for `#[cfg(mobile)]`:
```rust
#[cfg(desktop)]
let builder = builder.plugin(tauri_plugin_autostart::init(
    tauri_plugin_autostart::MacosLauncher::LaunchAgent,
    None,
));
```

**Call site 1 — setup prefs load** (line 64) — replace `default_prefs_path()` with the AppHandle-aware resolver:
```rust
// BEFORE (line 64):
let prefs_path = prefs::default_prefs_path();
// AFTER:
let prefs_path = prefs::prefs_path_for_app(app.handle());
```

**Call site 2 — setup state load** (lines 72–83) — replace `default_state_path()`:
```rust
// BEFORE (line 72):
let save_path = persistence::default_state_path();
// AFTER:
let save_path = persistence::state_path_for_app(app.handle());
// Lines 73-83 unchanged (error-handling pattern kept exactly):
let initial_state = match persistence::load_state(&save_path) {
    Ok(state) => state,
    Err(e) if save_path.exists() => {
        eprintln!(
            "hp41-gui: state load failed for {} ({e}); starting fresh",
            save_path.display()
        );
        hp41_core::CalcState::new()
    }
    Err(_) => hp41_core::CalcState::new(),
};
```

**Call site 3 — auto-save thread** (lines 102–116) — the path must be resolved BEFORE the `std::thread::spawn` and move-captured:
```rust
// BEFORE (line 102-116):
let handle = app.handle().clone();
std::thread::spawn(move || {
    let thread_save_path = persistence::default_state_path();  // ← PROBLEM: no AppHandle
    loop {
        std::thread::sleep(std::time::Duration::from_secs(30));
        let state = handle.state::<AppState>();
        let snapshot = state.lock().unwrap_or_else(|e| e.into_inner()).clone();
        if let Err(e) = persistence::save_state(&thread_save_path, &snapshot) {
            eprintln!("auto-save failed: {e}");
        }
    }
});
// AFTER: resolve path before spawn, then move-capture the PathBuf:
let handle = app.handle().clone();
let thread_save_path = persistence::state_path_for_app(&handle); // resolved here
std::thread::spawn(move || {
    loop {
        std::thread::sleep(std::time::Duration::from_secs(30));
        let state = handle.state::<AppState>();
        let snapshot = state.lock().unwrap_or_else(|e| e.into_inner()).clone();
        if let Err(e) = persistence::save_state(&thread_save_path, &snapshot) {
            eprintln!("auto-save failed: {e}");
        }
    }
});
```

**Key detail:** `AppHandle` is available in `setup()` via `app.handle()` — the `Manager` trait is already imported at line 4 (`use tauri::Manager`). `AppHandle` is `Clone + Send`, so cloning before the spawn is the established pattern (line 102 already does this).

---

### `hp41-gui/src-tauri/src/commands.rs` (controller, request-response)

**Analog:** `commands::import_raw_dialog` (lines 619–623) — existing command that already takes `app: AppHandle` as a parameter.

**Existing AppHandle-parameter pattern** (lines 618–622):
```rust
#[tauri::command]
pub fn import_raw_dialog(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<ImportRawResponse, GuiError> {
```

**Call site 3 — `save_state` command** (lines 558–564) — add `AppHandle` parameter:
```rust
// BEFORE:
#[tauri::command]
pub fn save_state(state: State<'_, AppState>) -> Result<(), String> {
    let snapshot = state.lock().unwrap_or_else(|e| e.into_inner()).clone();
    let path = persistence::default_state_path();
    persistence::save_state(&path, &snapshot).map_err(|e| e.to_string())
}
// AFTER:
#[tauri::command]
pub fn save_state(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    // CR-01: clone under lock, then release lock before disk I/O.
    let snapshot = state.lock().unwrap_or_else(|e| e.into_inner()).clone();
    let path = persistence::state_path_for_app(&app);
    persistence::save_state(&path, &snapshot).map_err(|e| e.to_string())
}
```

**Call site 4 — `set_pref` command** (lines 493–522) — add `AppHandle` parameter:
```rust
// BEFORE (lines 494-522):
#[tauri::command]
pub fn set_pref(
    key: String,
    value: String,
    prefs: State<'_, PrefsState>,
) -> Result<(), String> {
    // ... validation ...
    save_prefs(&default_prefs_path(), &*p).map_err(|e| e.to_string())
}
// AFTER:
#[tauri::command]
pub fn set_pref(
    app: AppHandle,
    key: String,
    value: String,
    prefs: State<'_, PrefsState>,
) -> Result<(), String> {
    // ... validation unchanged ...
    save_prefs(&prefs::prefs_path_for_app(&app), &*p).map_err(|e| e.to_string())
}
```

**Import update** — `default_prefs_path` is currently imported at line 21; after the change it is no longer needed from commands.rs (the path resolution moves to the `prefs_path_for_app` function). Update the import:
```rust
// BEFORE (line 21):
use crate::prefs::{default_prefs_path, save_prefs, GuiPrefs, VALID_LAUNCH_MODES, VALID_THEMES};
// AFTER:
use crate::prefs::{save_prefs, GuiPrefs, VALID_LAUNCH_MODES, VALID_THEMES};
```

**Note:** `AppHandle` is already imported at line 31 (`use tauri::AppHandle`) — no new import needed.

---

### `hp41-gui/src/App.tsx` (component, event-driven)

**Analog:** Lines 932–936 — the `keydown` useEffect with `addEventListener` / `removeEventListener` cleanup.

**Existing keydown listener pattern** (lines 932–936) — copy this structure exactly:
```typescript
// Register keyboard listener — cleanup required for React StrictMode (D-12)
useEffect(() => {
  window.addEventListener('keydown', handleKey);
  return () => window.removeEventListener('keydown', handleKey);
}, [handleKey]);
```

**New visibilitychange listener** — place immediately after the keydown useEffect:
```typescript
// Phase 54 PERSIST-02: save state when the app is backgrounded (iOS resign-active).
// visibilitychange fires in WKWebView when the user presses the Home button or
// switches apps. Fire-and-forget: the 30s auto-save thread (D-54.2a) is the safety net.
// Empty deps: handler has no dependency on React state — invoke always saves current state.
useEffect(() => {
  const handleVisibilityChange = () => {
    if (document.visibilityState === 'hidden') {
      void invoke<void>('save_state').catch((err: unknown) => {
        // Silent failure acceptable: the 30s timer is the safety net (D-54.2a)
        console.warn('background save failed:', extractErrMessage(err));
      });
    }
  };
  document.addEventListener('visibilitychange', handleVisibilityChange);
  return () => document.removeEventListener('visibilitychange', handleVisibilityChange);
}, []); // empty deps: listener is stable, registered once on mount
```

**Key details:**
- `invoke` is already imported at line 2: `import { invoke } from '@tauri-apps/api/core'`
- `extractErrMessage` is defined at lines 75–85 — available in module scope
- `void invoke<void>(...)` pattern is established at line 116: `void invoke<void>('request_cancel')`
- The `return () => removeEventListener(...)` cleanup prevents double-registration under React StrictMode — same rationale as the keydown cleanup at line 935

---

### `hp41-gui/src-tauri/tests/fixtures/v40-autosave.json` (test fixture)

**Analog:** `hp41-core/tests/fixtures/v33-autosave.json` — but the GUI fixture must use the `StateFile` wrapper (`{"version":1,"state":{…}}`) because `persistence::load_state()` deserializes `StateFile`, not bare `CalcState`.

**Core fixture format** (bare `CalcState`, NOT suitable for GUI tests):
```json
{
  "stack": { "x": "0", "y": "0", "z": "0", "t": "0", "lastx": "0", "lift_enabled": true },
  "regs": ["0", ...],
  ...
}
```

**GUI fixture format required** (with `StateFile` wrapper):
```json
{
  "version": 1,
  "state": {
    "stack": { "x": "0", "y": "0", "z": "0", "t": "0", "lastx": "0", "lift_enabled": true },
    "regs": [...],
    "xrom_modules": 31,
    "rand_seed": "3.14159265",
    "adv_tvm_state": { ... },
    "xmem_files": [ { "name": "DATFILE", "registers": ["1.5", "2.0"] } ],
    "xmem_active_file": "DATFILE",
    ...
  }
}
```

**Generation strategy** — use the `temp_path` + `save_state` pattern (lines 81–89 in persistence.rs) to generate JSON programmatically, then commit. The v4.0-era fields to set explicitly:
- `xrom_modules: 0b0001_1111` (31) — all 5 XROM modules enabled
- `rand_seed` — non-zero `HpNum` (e.g. `"3.14159265"`)
- `adv_tvm_state` — `Some(TvmState { ... })` (two-exception policy, Pitfall 20 guard)
- `xmem_files` — at least one `XmemFile { name: "DATFILE", registers: vec!["1.5", "2.0"] }`
- `xmem_active_file` — `Some("DATFILE".to_string())`

**Test that consumes the fixture** — add to `persistence.rs` `#[cfg(test)]` mod, modeled on `test_loads_v1_format_state_file` (lines 157–184):
```rust
#[test]
fn test_loads_v40_autosave_fixture() {
    let fixture = include_str!("../../tests/fixtures/v40-autosave.json");
    let path = temp_path("v40_compat");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, fixture.as_bytes()).unwrap();
    let loaded = load_state(&path).expect("v4.0-format save must load");
    assert!(!loaded.is_running, "is_running must be false after load");
    assert_eq!(loaded.xrom_modules, 0b0001_1111u8, "v4.0 xrom_modules preserved");
    // Verify non-empty xmem_files (v4.0 X-MEM state survives round-trip)
    assert!(!loaded.xmem_files.is_empty(), "xmem_files must be non-empty in v4.0 fixture");
    let _ = fs::remove_dir_all(path.parent().unwrap());
}
```

**`include_str!` path** — from `src/persistence.rs`, the fixture in `tests/fixtures/v40-autosave.json` resolves as `../../tests/fixtures/v40-autosave.json` (relative to the source file).

---

## Shared Patterns

### `#[cfg(desktop)]` / `#[cfg(mobile)]` gate
**Source:** `hp41-gui/src-tauri/src/lib.rs` lines 46–50
**Apply to:** `persistence.rs` `state_path_for_app()`, `prefs.rs` `prefs_path_for_app()`
```rust
#[cfg(desktop)]
let builder = builder.plugin(tauri_plugin_autostart::init(
    tauri_plugin_autostart::MacosLauncher::LaunchAgent,
    None,
));
```

### Mutex clone-before-I/O (CR-01)
**Source:** `hp41-gui/src-tauri/src/lib.rs` lines 110–112
**Apply to:** `commands::save_state`, auto-save thread in `lib.rs`
```rust
let state = handle.state::<AppState>();
let snapshot = state.lock().unwrap_or_else(|e| e.into_inner()).clone();
if let Err(e) = persistence::save_state(&thread_save_path, &snapshot) {
    eprintln!("auto-save failed: {e}");
}
```

### Poisoned-lock recovery
**Source:** `hp41-gui/src-tauri/src/commands.rs` line 499 and lib.rs line 111
**Apply to:** All Mutex lock sites in commands.rs
```rust
prefs.lock().unwrap_or_else(|e| e.into_inner())
state.lock().unwrap_or_else(|e| e.into_inner())
```

### React fire-and-forget invoke with `.catch()`
**Source:** `hp41-gui/src/App.tsx` lines 313–315
**Apply to:** `visibilitychange` handler invoke call
```typescript
invoke('set_pref', { key: 'theme', value: newTheme }).catch(() => {
  // Persistence failure is non-fatal — theme applies visually regardless.
});
```

### React useEffect addEventListener cleanup
**Source:** `hp41-gui/src/App.tsx` lines 933–936
**Apply to:** `visibilitychange` useEffect
```typescript
useEffect(() => {
  window.addEventListener('keydown', handleKey);
  return () => window.removeEventListener('keydown', handleKey);
}, [handleKey]);
```

### `temp_path` test helper + `create_dir_all` + `remove_dir_all` cleanup
**Source:** `hp41-gui/src-tauri/src/persistence.rs` lines 74–78, 84, 89
**Apply to:** New tests `test_loads_v40_autosave_fixture` and `test_state_path_for_app_mobile`
```rust
fn temp_path(name: &str) -> PathBuf {
    std::env::temp_dir()
        .join(format!("hp41_test_{name}"))
        .join("state.json")
}
// Usage:
fs::create_dir_all(path.parent().unwrap()).unwrap();
// ... test body ...
let _ = fs::remove_dir_all(path.parent().unwrap());
```

---

## No Analog Found

All files have close analogs. No entries.

---

## Call Site Inventory (all four must be updated atomically per Pitfall 1)

| # | File | Line | Current Call | Replace With |
|---|------|------|--------------|--------------|
| 1 | `lib.rs` | 64 | `prefs::default_prefs_path()` | `prefs::prefs_path_for_app(app.handle())` |
| 2 | `lib.rs` | 72 | `persistence::default_state_path()` | `persistence::state_path_for_app(app.handle())` |
| 3 | `lib.rs` | 104 | `persistence::default_state_path()` inside thread | resolve to `PathBuf` before spawn; move-capture |
| 4 | `commands.rs` | 562 | `persistence::default_state_path()` | `persistence::state_path_for_app(&app)` (add `app: AppHandle` param) |
| 5 | `commands.rs` | 521 | `default_prefs_path()` | `prefs::prefs_path_for_app(&app)` (add `app: AppHandle` param) |

---

## Metadata

**Analog search scope:** `hp41-gui/src-tauri/src/`, `hp41-gui/src/`, `hp41-core/tests/fixtures/`
**Files read:** 7 source files + 1 fixture file
**Pattern extraction date:** 2026-06-02
