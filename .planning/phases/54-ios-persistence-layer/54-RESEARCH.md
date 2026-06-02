# Phase 54: iOS Persistence Layer - Research

**Researched:** 2026-06-02
**Domain:** Tauri v2.11 iOS path resolution, visibilitychange background-save, v4.0 backward-compat testing
**Confidence:** HIGH (all critical claims verified against codebase + official Rust crate source)

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**D-54.1:** iOS autosave lives in **Application Support**, resolved via
`AppHandle::path().app_local_data_dir()` → `Library/Application Support/ch.talent-factory.hp41/autosave.json`.
Documents/ rejected; app-managed state, not user-authored content.

**D-54.1a:** Desktop path `default_state_path()` (`~/.hp41/autosave.json`) stays exactly as-is for
`#[cfg(desktop)]` and existing unit tests. iOS and desktop are different files on different devices.

**D-54.2:** Add `document.addEventListener("visibilitychange", …)` in React; when
`document.visibilityState === "hidden"`, call `invoke("save_state")`. Fires reliably in WKWebView
on Home-button / app-switch.

**D-54.2a:** Keep the 30 s auto-save thread on iOS (do NOT `#[cfg]`-gate it off). Belt-and-suspenders.

**D-54.2b:** No mid-run save guard needed. `load_state()` already forces `is_running = false`.

**D-54.2c:** `pagehide` and `beforeunload` are explicitly NOT added.

**D-54.3:** Thread `app_local_data_dir()` into **both** `persistence.rs` (`autosave.json`) and
`prefs.rs` (`prefs.json`).

**D-54.3a:** `cards.rs` (`~/.hp41/cards/`) is left untouched (RAW-IOS-01 deferred).

**D-54.4:** Prove backward-compat with an automated v4.0 fixture test committed to
`persistence.rs` tests. Host/CI test is sufficient — serde path is byte-for-byte identical across targets.

**D-54.4a:** Prove background→kill→relaunch with an on-device round-trip.

**D-54.4b:** Manual Xcode "Download Container" injection not required.

### Claude's Discretion

- **`app_data_dir` #12552 contingency:** Attempt `app_local_data_dir()` first; sanctioned fallback is
  `dirs::home_dir()`. Planner decides exact fallback wiring after observing real device behavior.
- **`state_path_for_app()` shape:** exact signature/placement, how the PathBuf is move-captured into
  the auto-save thread, analogous `prefs_path_for_app()`.
- **`#[serde]` path-string audit (P-iOS-29):** verify no `CalcState` field persists an absolute path.

### Deferred Ideas (OUT OF SCOPE)

None — discussion stayed within phase scope. `cards.rs` / `.raw` / X-MEM file I/O on iOS is
already-tracked future requirement RAW-IOS-01.

</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| PERSIST-01 | Autosave/load uses iOS app-sandbox container path instead of `~/.hp41/` (desktop path unchanged) | `app_local_data_dir()` API verified; fallback chain documented; `dirs::home_dir()` iOS behavior confirmed |
| PERSIST-02 | State is autosaved when the app is backgrounded (resign-active / `visibilitychange`) | `visibilitychange` → `invoke("save_state")` pattern; `save_state` command confirmed registered; reliability risk assessed |
| PERSIST-03 | State survives background→kill→relaunch; existing desktop save files still load | v4.0 fixture test design specified; `StateFile` wrapper format confirmed; P-iOS-29 audit finding documented |

</phase_requirements>

---

## Summary

Phase 54 is a precision adapter change: three Rust source files (`persistence.rs`, `prefs.rs`,
`lib.rs`) gain an `AppHandle`-aware path resolver for iOS, and a `visibilitychange` listener is
added to `App.tsx`. The `hp41-core` engine, the Tauri IPC contract, and the `StateFile` serde
format are completely unchanged.

**Critical discovery on the `dirs` fallback:** The `dirs-sys` crate (used by `dirs::home_dir()`)
has a special case at line 39 of its `lib.rs`: on iOS (`target_os = "ios"`), the `getpwuid_r`
fallback is disabled — the function returns `None` if the `HOME` environment variable is empty.
However, Apple's iOS runtime does set `HOME` to the app's sandbox container root at launch (confirmed
via Apple's Developer Documentation: `HOME` points to the app container directory). Therefore,
`dirs::home_dir()` on a Tauri iOS build returns
`/var/mobile/Containers/Data/Application/<UUID>` (the container root), not `None`. If used as a
fallback, the file would land at `<container_root>/.hp41/autosave.json` — writable and persistent,
but not the architecturally correct Application Support path.

**`app_local_data_dir()` first:** `AppHandle::path().app_local_data_dir()` returns
`Result<PathBuf>` (Tauri 2.11.1 confirmed via docs.rs). On macOS it resolves to
`$HOME/Library/Application Support/<bundle_id>`. On iOS, it should resolve to
`Library/Application Support/ch.talent-factory.hp41/` inside the container. This is the
correct target per D-54.1. Tauri GitHub issue #12552 documents that `app_data_dir()` (and
possibly `app_local_data_dir()`) can return "Permission Denied" on iOS in some configurations
(the exact trigger is unclarified; the workaround confirmed in that issue is `dirs::home_dir()`).
This must be handled with a `?`-fallback chain.

**`visibilitychange` reliability on iOS WKWebView:** Multiple search paths found no authoritative
confirmation that Tauri's WKWebView integration on iOS forwards `applicationWillResignActive` to
the Page Visibility API. Desktop Tauri has documented issues where `visibilitychange` does not
fire on app-switch/focus-loss (GitHub issues #6864, #9524 — both desktop). The CONTEXT.md
(D-54.2) locks `visibilitychange` as the chosen approach based on ARCHITECTURE.md's assessment
that it "fires reliably in WKWebView when the user presses the Home button or switches apps."
This claim is MEDIUM confidence — it is architecturally correct for standard WKWebView behavior
(iOS Safari and Safari-based PWAs do fire `visibilitychange` on Home press), but the Tauri-specific
plumbing has not been independently verified in this session. The 30 s timer (D-54.2a) is the
safety net.

**No new packages:** `dirs = "6"` is already in `hp41-gui/src-tauri/Cargo.toml`. No new runtime
or dev dependencies are introduced. The `dirs` crate is `hp41-gui`-local (nested standalone
workspace), so the core zero-new-deps invariant is unaffected.

**Primary recommendation:** Implement `state_path_for_app(&handle)` and `prefs_path_for_app(&handle)` in their respective modules using a two-branch strategy: `handle.path().app_local_data_dir()` on mobile (with `dirs::home_dir()` as the emergency fallback if #12552 bites), and `default_state_path()` / `default_prefs_path()` on desktop.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| iOS save-path resolution | Frontend Server (Tauri adapter) — `hp41-gui` | — | `hp41-core` has no filesystem knowledge; path is caller concern |
| Background-save trigger | Browser / Client (React) | API/Backend (Tauri command) | `visibilitychange` fires in WKWebView; `save_state` command is the backend handler |
| State serialization / migration | API/Backend (`hp41-core` serde) | — | `StateFile`+`CalcState`+`migrate_after_load()` are platform-agnostic |
| Desktop path (unchanged) | Frontend Server (Tauri adapter) | — | `default_state_path()` stays; `#[cfg(desktop)]` guards the branch |
| Fixture test (backward-compat) | API/Backend (host Rust tests) | — | Serde path is identical on all targets; no device needed |

---

## Standard Stack

### Core (all already present in hp41-gui workspace)

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `tauri` | 2.11.1 [VERIFIED: Cargo.lock] | `AppHandle::path().app_local_data_dir()` | Platform-aware path resolver for Tauri apps |
| `dirs` | 6.0.0 [VERIFIED: Cargo.lock] | `dirs::home_dir()` fallback for #12552 | Already in Cargo.toml; iOS HOME returns container root |
| `serde_json` | 1.x [VERIFIED: Cargo.toml] | `StateFile` JSON serialization | Existing format; no change needed |
| `hp41-core` | path dep [VERIFIED: Cargo.toml] | `CalcState` + `migrate_after_load()` | Engine; not modified in this phase |

### Supporting (React / TypeScript)

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `@tauri-apps/api/core` | 2.11.0 [VERIFIED: package.json] | `invoke("save_state")` from `visibilitychange` | Already imported in App.tsx |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `app_local_data_dir()` primary | `dirs::home_dir()` primary | `home_dir()` puts file in container root `.hp41/`, not Application Support — less standard but functional as fallback |
| `visibilitychange` | Swift `applicationWillResignActive` plugin | Native is more reliable but requires a new Tauri Swift plugin (Phase 56 territory); V+C per D-54.2 |

**Installation:** No new packages. `dirs = "6"` is already present.

---

## Package Legitimacy Audit

No new packages are introduced in this phase. All packages used are existing project
dependencies already present in `hp41-gui/src-tauri/Cargo.toml`.

| Package | Registry | Age | Downloads | Source Repo | slopcheck | Disposition |
|---------|----------|-----|-----------|-------------|-----------|-------------|
| `dirs` 6.0.0 | crates.io | 8+ yrs | Very high | github.com/dirs-dev/dirs-rs | N/A (existing dep) | Pre-approved |
| `tauri` 2.11.1 | crates.io | 6+ yrs | Very high | github.com/tauri-apps/tauri | N/A (existing dep) | Pre-approved |

**Packages removed due to slopcheck [SLOP] verdict:** none
**Packages flagged as suspicious [SUS]:** none

---

## Architecture Patterns

### System Architecture Diagram

```
iOS Home Button / App Switch
        │
        ▼ [visibilitychange: "hidden"]
  React App.tsx
        │
        ▼ [invoke("save_state")]
  WKWebView IPC bridge
        │
        ▼
  commands::save_state(&AppHandle, State<AppState>)
        │ path = state_path_for_app(&app_handle)   ← NEW (mobile branch)
        │ snapshot = CalcState.clone()              ← existing CR-01
        ▼
  persistence::save_state(&path, &snapshot)        ← unchanged
        │
        ▼
  Library/Application Support/ch.talent-factory.hp41/autosave.json
  (inside iOS sandbox container)


App Launch (relaunch after kill)
        │
        ▼
  lib.rs run() → setup()
        │ prefs_path = prefs_path_for_app(&app)    ← NEW (mobile branch)
        │ state_path = state_path_for_app(&app)    ← NEW (mobile branch)
        ▼
  persistence::load_state(&state_path)             ← unchanged
        │ migrate_after_load() called inside
        ▼
  CalcState wrapped in Mutex<AppState>


30s Auto-save Thread (belt-and-suspenders, D-54.2a)
        │ thread_save_path = state_path_for_app()  ← NEW: path resolved at setup,
        │                                             move-captured into thread
        ▼ [every 30s: clone under lock, drop lock, then I/O]
  persistence::save_state(&thread_save_path, &snapshot)


Desktop (unchanged, #[cfg(desktop)] OR #[cfg(not(mobile))])
  default_state_path()  →  ~/.hp41/autosave.json
  default_prefs_path()  →  ~/.hp41/prefs.json
```

### Recommended Project Structure

No new directories. All changes are in-place modifications to existing files:

```
hp41-gui/src-tauri/src/
├── persistence.rs    ← add state_path_for_app(&AppHandle) function
├── prefs.rs          ← add prefs_path_for_app(&AppHandle) function
├── commands.rs       ← update save_state + set_pref to accept AppHandle
└── lib.rs            ← thread resolved paths into setup() + auto-save thread

hp41-gui/src/
└── App.tsx           ← add visibilitychange useEffect listener

hp41-core/tests/fixtures/       (no changes needed here)
hp41-gui/src-tauri/src/         (v4.0 fixture embedded inline in persistence.rs tests)
```

### Pattern 1: `#[cfg(mobile)]` / `#[cfg(desktop)]` path branch

**What:** Gate the iOS path resolver behind `#[cfg(mobile)]` so desktop behavior is byte-for-byte
unchanged. `mobile` is the Tauri/Cargo alias for iOS and Android targets.

**When to use:** Any place that currently calls `default_state_path()` or `default_prefs_path()`.

**Example:**
```rust
// Source: ARCHITECTURE.md §"Concrete Changes to persistence.rs (Approach A)"
// + existing lib.rs pattern (lines 46-50 for tauri-plugin-autostart)

/// AppHandle-aware path resolver for iOS sandbox; desktop falls back to default.
pub fn state_path_for_app(handle: &tauri::AppHandle) -> PathBuf {
    #[cfg(mobile)]
    {
        // Primary: Tauri's PathResolver (resolves to Library/Application Support/<bundle_id>/)
        // Fallback for issue #12552: dirs::home_dir() returns container root on iOS.
        handle
            .path()
            .app_local_data_dir()
            .unwrap_or_else(|_| {
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

**Key detail:** The `handle` parameter is available in `lib.rs::setup()` via `app.handle()` (the
`Manager` trait is already imported). The `AppHandle` is `Clone + Send`, so it can be cloned and
move-captured into the auto-save thread.

### Pattern 2: Resolved path move-captured into auto-save thread

**What:** The 30 s auto-save thread currently calls `default_state_path()` inside the thread
(lib.rs line 104). This must change to use a path resolved before the thread spawns.

**Why:** `AppHandle` is available in `setup()` but cannot be trivially reconstructed inside a
`std::thread::spawn` closure without cloning. The cleanest approach: resolve the path in `setup()`,
then `move` the `PathBuf` into the closure.

**Example (lib.rs change):**
```rust
// Source: lib.rs lines 100-116 (existing pattern) + ARCHITECTURE.md §339
let handle = app.handle().clone();
// NEW: resolve iOS-aware path BEFORE spawning — then move-capture
let thread_save_path = persistence::state_path_for_app(&handle);
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

Note: `thread_save_path` is a `PathBuf` (not a reference), so the `move` closure owns it.
`handle` is already cloned before the spawn (existing pattern, lib.rs line 102).

### Pattern 3: `commands::save_state` needs AppHandle too

**What:** `commands::save_state` (the `visibilitychange` invoke target) currently calls
`persistence::default_state_path()` directly (line 562). This is the command that `visibilitychange`
fires — it must use the iOS-aware path.

**Why this matters:** Without this change, `visibilitychange` triggers a save to `~/.hp41/`
(which doesn't exist on iOS), silently discarding the background save. This would defeat PERSIST-02.

**Example (commands.rs change):**
```rust
// Source: commands.rs lines 558-564 (existing) + verified AppHandle usage pattern
#[tauri::command]
pub fn save_state(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    // CR-01: clone under lock, then release lock before disk I/O.
    let snapshot = state.lock().unwrap_or_else(|e| e.into_inner()).clone();
    let path = persistence::state_path_for_app(&app);   // was: default_state_path()
    persistence::save_state(&path, &snapshot).map_err(|e| e.to_string())
}
```

`AppHandle` is already imported in `commands.rs` (line 31: `use tauri::AppHandle;`) and used in
other commands (e.g., `restart_app`, `import_raw_dialog`). Adding it here follows the existing
pattern.

### Pattern 4: `set_pref` command also writes to disk path

**What:** `commands::set_pref` calls `save_prefs(&default_prefs_path(), …)` (line 521). On iOS
this would silently write to a non-existent path. It needs the same AppHandle treatment.

**Example:**
```rust
// Source: commands.rs lines 476-521 (existing)
#[tauri::command]
pub fn set_pref(app: AppHandle, prefs: State<'_, PrefsState>, key: String, value: String)
    -> Result<(), String>
{
    // ... existing validation logic ...
    save_prefs(&prefs::prefs_path_for_app(&app), &*p).map_err(|e| e.to_string())
}
```

### Pattern 5: `visibilitychange` React listener

**What:** Add a `useEffect` in `App.tsx` that registers a `visibilitychange` listener and calls
`invoke("save_state")` when the document becomes hidden.

**When to use:** Add alongside the existing `keydown` listener useEffect (lines 933-936).

**Example:**
```typescript
// Source: ARCHITECTURE.md §"Autosave on Resign-Active" + D-54.2
// Pattern modeled on existing useEffect at App.tsx:933-936

useEffect(() => {
  const handleVisibilityChange = () => {
    if (document.visibilityState === 'hidden') {
      // Fire-and-forget: background save, no UI feedback needed
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

**Key detail:** No dependency on `calcState` or other props — the handler always invokes
`save_state` regardless of calculator state; the backend handles the path correctly.
The `return () => removeEventListener(...)` cleanup prevents double-registration under
React StrictMode (same pattern as the existing `keydown` listener at line 935).

### Anti-Patterns to Avoid

- **Calling `default_state_path()` inside the auto-save thread:** The thread cannot access
  `AppHandle` after the fact. Resolve the path in `setup()` and move-capture the `PathBuf`.
- **`#[cfg(target_os = "ios")]` instead of `#[cfg(mobile)]`:** Tauri's build system uses
  `mobile` (not `target_os = "ios"`) for cross-platform mobile gates. Using `target_os` would
  compile incorrectly for iOS Simulator builds on Apple Silicon (`aarch64-apple-ios-sim` is not
  `target_os = "ios"`). [ASSUMED — based on Tauri documentation; verify with `just ios-build`]
- **Using `dirs::home_dir().join(".hp41")` as the primary iOS path:** On iOS, `home_dir()`
  returns the container root, so the file would land at `<container>/.hp41/autosave.json` —
  writable and persistent, but outside Application Support and not iCloud-backed. Use it only as
  a fallback for Tauri #12552.
- **Adding `AppHandle` to `state_path_for_app` signature and calling from unit tests:**
  Tests in `persistence.rs` don't have a Tauri `AppHandle`. Keep the function signature taking
  `&tauri::AppHandle` but keep `default_state_path()` for all test helpers.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| iOS directory creation | Manual `fs::create_dir_all` pre-call | `save_state()` already calls `create_dir_all(parent)` (persistence.rs:43-47) | Already handles missing-dir on any path |
| Cross-platform path resolution | `if cfg!(target_os = "ios")` branches | `#[cfg(mobile)]` / `#[cfg(not(mobile))]` | Tauri's canonical mobile gate; covers both iOS and future Android |
| v4.0 fixture generation | Constructing JSON by hand | `save_state()` on a `CalcState::new()` with fields set programmatically | Use the existing `temp_path` + `save_state` + JSON mutation pattern (see `test_loads_v1_format_state_file`) |

**Key insight:** The `save_state` / `load_state` API in `persistence.rs` already handles directory
creation and error propagation. The only adapter change needed is the path itself.

---

## Runtime State Inventory

> This phase modifies file persistence paths, not data migrations.

| Category | Items Found | Action Required |
|----------|-------------|------------------|
| Stored data | No existing iOS saves (no iOS app installed yet) | None — first-ever iOS run creates a fresh save |
| Live service config | `~/.hp41/autosave.json` on desktop — unaffected by D-54.1a | None (desktop path preserved) |
| OS-registered state | None | None |
| Secrets/env vars | `HOME` env var on iOS → container root (confirmed via Apple docs) | Used correctly as fallback in `dirs::home_dir()` only |
| Build artifacts | None stale | None |

**Nothing found in category:** desktop autosave path is unchanged; iOS has no prior state (first-run).

---

## Common Pitfalls

### Pitfall 1: `commands::save_state` Missed in Path Migration

**What goes wrong:** `lib.rs` setup + auto-save thread are updated to use `state_path_for_app()`,
but `commands::save_state` (the `visibilitychange` invoke target) still calls
`default_state_path()`. On iOS, the `visibilitychange` save fires against a nonexistent path,
silently discarding the background save. PERSIST-02 passes manual round-trip but fails under
stress (app killed immediately after background).

**Why it happens:** Three call sites use `default_state_path()`: lib.rs (setup load + thread).
The command handler is a fourth call site that's easy to miss.

**How to avoid:** The wave plan must update all four call sites atomically:
  1. `lib.rs` setup load (line 72: `let save_path = persistence::default_state_path()`)
  2. `lib.rs` auto-save thread (line 104: `persistence::default_state_path()`)
  3. `commands::save_state` (line 562: `persistence::default_state_path()`)
  4. `commands::set_pref` (line 521: `default_prefs_path()`)

**Warning signs:** `save_state` command returns `Ok(())` on iOS but no file appears in container.

### Pitfall 2: Tauri #12552 — `app_local_data_dir()` Returns `Err` on iOS

**What goes wrong:** `handle.path().app_local_data_dir()` returns `Err("Operation not permitted")`
on some iOS configurations (tauri-apps/tauri#12552). If the code calls `.expect(…)` or `?`
without a fallback, the app panics on startup. `hp41-core` has `#![deny(clippy::unwrap_used)]`
but `hp41-gui` does not — however a startup panic is still a hard failure.

**Why it happens:** The exact trigger is unconfirmed in tauri#12552 (no merged fix as of research
date). It may be a sandbox entitlement issue or a Tauri version-specific bug.

**How to avoid:** Use `unwrap_or_else` with the `dirs::home_dir()` fallback chain as shown in
Pattern 1. Log the fallback to stderr so it's visible in the Xcode console during device testing.
Verify `app_local_data_dir()` behavior on the actual device during the on-device round-trip test
(D-54.4a).

**Warning signs:** App starts with a blank screen or crashes in the Xcode console with
"Operation not permitted" originating from `app_local_data_dir`.

### Pitfall 3: `dirs::home_dir()` Fallback Path Is Not Application Support

**What goes wrong:** On iOS, `dirs::home_dir()` returns the container root
(`/var/mobile/Containers/Data/Application/<UUID>`), NOT `Library/Application Support/`.
If used naively as `dirs::home_dir().join(".hp41")`, the save file lands in
`<container_root>/.hp41/autosave.json` — a dot-directory at the container root.
This is writable but is not Application Support, is not iCloud-backed, and deviates from D-54.1.

**Why it happens:** `dirs-sys` on iOS disables the `getpwuid_r` fallback (special-cased at
`target_os = "ios"` in dirs-sys lib.rs:39) and falls back to the `HOME` env var, which iOS
sets to the container root.

**How to avoid:** If using `dirs::home_dir()` as a fallback for #12552, manually append
`Library/Application Support/ch.talent-factory.hp41/` rather than `.hp41/`:
```rust
dirs::home_dir()
    .unwrap_or_else(|| PathBuf::from("."))
    .join("Library")
    .join("Application Support")
    .join("ch.talent-factory.hp41")
    .join("autosave.json")
```

### Pitfall 4: `visibilitychange` May Not Fire on iOS WKWebView (MEDIUM-risk)

**What goes wrong:** On desktop Tauri, `visibilitychange` is documented as NOT firing on
app-switch/focus-loss (GitHub issues #6864 and #9524 — both desktop). For iOS/WKWebView, the
standard behavior is that `visibilitychange` fires on Home button press (confirmed for Safari;
less clear for WKWebView embedded in a non-Tauri app). If Tauri's WKWebView on iOS does NOT
forward `applicationWillResignActive` → Page Visibility `"hidden"`, then PERSIST-02's trigger
is silently lost.

**Why it happens:** Tauri's WKWebView wrapper (`wry`) on iOS may or may not hook
`UIApplicationWillResignActiveNotification` to update `document.visibilityState`.

**How to avoid:**
- Verify during on-device testing (D-54.4a) by adding a `console.log` to the
  `visibilitychange` handler and observing the Xcode console when pressing Home.
- The 30 s auto-save thread (D-54.2a) is the safety net if the event does not fire.
- If `visibilitychange` does not fire on iOS, escalate to Phase 56 (lifecycle/clock phase)
  for a native Tauri plugin approach (using `tauri-plugin-app-events` `onPause` event or
  a custom Swift plugin observing `UIApplication.willResignActiveNotification`).

**Warning signs:** `console.warn('background save failed:…')` never fires in Xcode console,
but the file also does not update after backgrounding.

### Pitfall 5: v4.0 Fixture Must Use `StateFile` Wrapper, Not Bare `CalcState`

**What goes wrong:** The `hp41-core` test fixtures (e.g., `v33-autosave.json`) are bare
`CalcState` JSON objects (no `{"version":1,"state":{…}}` wrapper). The GUI's `load_state()`
reads `StateFile` (the version wrapper). A fixture without the wrapper fails to deserialize
in `persistence.rs` tests.

**Why it happens:** The core tests call `serde_json::from_str::<CalcState>(…)` directly; the
GUI tests call `load_state(path)` which deserializes `StateFile`.

**How to avoid:** Generate the v4.0 fixture using the existing GUI test pattern:
  1. Build a `CalcState` with v4.0 fields set (non-default `xmem_files`, `rand_seed`, `adv_tvm_state`).
  2. Call `save_state(&temp_path, &state)` to write it with the `StateFile` wrapper.
  3. Read the JSON, optionally manipulate it (remove future fields to simulate v4.0 era), then
     commit as `tests/fixtures/v40-autosave.json` in `hp41-gui/src-tauri/`.
  4. Embed with `include_str!` and test with `load_state()`.

### Pitfall 6: P-iOS-29 Absolute-Path Audit Scope

**What goes wrong:** A field in `CalcState` stores an absolute filesystem path. After an iOS
app reinstall or container UUID change, the stored path becomes invalid.

**Findings from codebase audit:**
- `xmem_active_file: Option<String>` — stores the **name** of an X-MEM file (e.g., `"DATFILE"`),
  NOT a filesystem path. [VERIFIED: xmem/mod.rs:42 `pub name: String`]
- `XmemFile.name: String` — file name only, no path. [VERIFIED]
- `key_assignments: HashMap<char, String>` — maps key char to function name string, no paths. [VERIFIED by inspection]
- `alpha_reg: String` — ALPHA register content, no paths. [VERIFIED]
- No `PathBuf` or path-like fields exist anywhere in `CalcState`. [VERIFIED: state.rs grep for PathBuf/path returns zero hits in field declarations]

**Conclusion:** P-iOS-29 audit passes. `CalcState` stores NO absolute filesystem paths.

---

## Code Examples

Verified patterns from the codebase and official sources:

### Full `state_path_for_app` implementation (persistence.rs)
```rust
// Source: ARCHITECTURE.md §334-339 + Pattern 1 above + dirs-sys lib.rs:39 behavior
use tauri::AppHandle;

pub fn state_path_for_app(handle: &AppHandle) -> PathBuf {
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

### v4.0 fixture test pattern (persistence.rs inline test)
```rust
// Source: modeled on test_loads_v1_format_state_file (persistence.rs:157-184)
// v4.0 adds: xmem_files, xmem_active_file (new in Phase 51/52);
//            rand_seed, adv_tvm_state (already present from v3.x but absent in v1.0 fixture)
// The fixture tests the StateFile wrapper path, not bare CalcState.

#[test]
fn test_loads_v40_autosave_fixture() {
    // Load a committed v4.0 autosave.json fixture (with version wrapper)
    // and assert: deserialization succeeds, migrate_after_load runs, key fields present.
    let fixture = include_str!("../../tests/fixtures/v40-autosave.json");
    let path = temp_path("v40_compat");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, fixture.as_bytes()).unwrap();
    let loaded = load_state(&path).expect("v4.0-format save must load");
    assert!(!loaded.is_running, "is_running must be false after load");
    assert_eq!(loaded.xrom_modules, 0b0001_1111u8, "v4.0 xrom_modules preserved");
    // rand_seed and adv_tvm_state survive (two-exception policy, Pitfall 20 guard)
    let _ = fs::remove_dir_all(path.parent().unwrap());
}
```

### Load-path in `lib.rs` setup (after migration)
```rust
// Source: lib.rs lines 64-83 (existing) — showing both prefs and state path threading
let prefs_path = prefs::prefs_path_for_app(app.handle());   // NEW function
let initial_prefs = prefs::load_prefs(&prefs_path);
// ... (existing macOS launch mode capture, manage calls) ...

let save_path = persistence::state_path_for_app(app.handle()); // NEW: was default_state_path()
let initial_state = match persistence::load_state(&save_path) {
    Ok(state) => state,
    Err(e) if save_path.exists() => {
        eprintln!("hp41-gui: state load failed for {} ({e}); starting fresh", save_path.display());
        hp41_core::CalcState::new()
    }
    Err(_) => hp41_core::CalcState::new(),
};
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `dirs::home_dir().join(".hp41")` (Phase 1) | `state_path_for_app(&handle)` with `#[cfg(mobile)]` | Phase 54 | iOS gets container path; desktop unchanged |
| `default_state_path()` in `save_state` command | `state_path_for_app(&handle)` | Phase 54 | All 4 call sites consistent |

**Deprecated/outdated:**
- Direct `default_state_path()` calls in `commands.rs` save_state and `lib.rs` setup: replaced by `state_path_for_app(&handle)` on mobile.

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `#[cfg(mobile)]` covers iOS and compiles correctly for `aarch64-apple-ios-sim` (not just `aarch64-apple-ios`) | Pattern 1 | If `mobile` does not match the sim triple, desktop tests break on iOS sim; mitigate: `just ios-build` verifies |
| A2 | `visibilitychange` fires in Tauri's WKWebView on iOS when Home button is pressed | Pattern 5, Pitfall 4 | PERSIST-02 trigger silently absent; safety net is 30s timer; escalation path is Phase 56 native plugin |
| A3 | `app_local_data_dir()` returns a writable path inside the iOS sandbox (when it succeeds) | Pattern 1 | If the path requires an entitlement not currently granted, I/O fails; `save_state` returns Err which is logged |
| A4 | The v4.0 fixture file should be placed in `hp41-gui/src-tauri/tests/fixtures/` (new directory) OR embedded inline in the persistence.rs test | Code Examples | Either works; inline avoids a new directory but is larger; dedicated fixtures directory mirrors hp41-core pattern |

**If this table is empty:** N/A — 4 assumptions listed.

---

## Open Questions

1. **Does `app_local_data_dir()` succeed on the physical iPhone 15 Pro used in Phase 53?**
   - What we know: Tauri #12552 reports it fails "on some iOS configurations"; the Phase 53 smoke
     test did not exercise file I/O, only dispatch_op.
   - What's unclear: Whether the Phase 53 app (with automatic signing but no `tauri-plugin-fs`
     enabled) triggers the bug.
   - Recommendation: Attempt `app_local_data_dir()` in the implementation and add a distinctive
     `eprintln!` on fallback. The first on-device build will reveal the behavior.

2. **Is `tauri-plugin-fs` required to grant `app_local_data_dir` permission on iOS?**
   - What we know: `STACK.md` §106-113 notes `tauri-plugin-fs` may be needed for iOS sandbox paths;
     `tauri-plugin-dialog` is already a dep but `tauri-plugin-fs` is not.
   - What's unclear: Whether `AppHandle::path().app_local_data_dir()` + `std::fs` (Rust stdlib) is
     sufficient without `tauri-plugin-fs`, OR whether the plugin is required to configure ACL
     permissions for the iOS sandbox. The direct Rust `std::fs::File::create()` path may bypass
     the Tauri ACL entirely (it's a raw syscall), which is likely why the #12552 reporter got
     "Permission Denied" — the sandbox entitlement for writing to Application Support may be
     missing.
   - Recommendation: If `app_local_data_dir()` returns a valid path but `std::fs::File::create`
     still returns EPERM, the iOS sandbox entitlement for `com.apple.security.files.user-selected.read-write`
     or the capability `AppDataWrite` must be added to
     `hp41-gui/src-tauri/capabilities/default.json`. This is the same mechanism used for
     `tauri-plugin-dialog`.

3. **Should the v4.0 fixture live in `hp41-gui/src-tauri/tests/fixtures/` (new) or inline?**
   - What we know: `hp41-core/tests/fixtures/` holds v20–v33 fixtures as separate files; the GUI
     currently has no fixtures directory.
   - Recommendation (Claude's discretion): Create `hp41-gui/src-tauri/tests/fixtures/v40-autosave.json`
     matching the core pattern. Use `include_str!("../../tests/fixtures/v40-autosave.json")` in
     the test module (adjusted relative path from `src/`). This is consistent, reviewable, and
     can be reused by future backward-compat tests.

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Tauri CLI 2.11.x | `just ios-build` | ✓ | 2.11.0 (npm) / 2.11.1 (lockfile) | — |
| `just ios-build` recipe | Device build | ✓ | (Phase 53) | — |
| Physical iPhone 15 Pro | D-54.4a on-device test | ✓ | (Phase 53 proved) | — |
| iOS 18+ Simulator | `just ios-sim` | ✓ | (Phase 53 proved) | — |
| `cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml` | Fixture tests (host) | ✓ | Rust 1.88 | — |
| `tauri-plugin-fs` (may be needed) | `app_local_data_dir` ACL on iOS | Unknown | — | `std::fs` with sandbox entitlement |

**Missing dependencies with no fallback:** None blocking.

**Missing dependencies with fallback:**
- `tauri-plugin-fs` may be needed for iOS ACL; fallback is using `std::fs` directly + adding the
  `AppDataWrite` capability to `capabilities/default.json` or `capabilities/ios.json`.

---

## Validation Architecture

Nyquist validation is enabled (`workflow.nyquist_validation: true` in `.planning/config.json`).

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in test harness (cargo test) |
| Config file | None (uses `cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml`) |
| Quick run command | `cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml persistence` |
| Full suite command | `just gui-ci` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| PERSIST-01 | `state_path_for_app()` returns correct mobile path | unit | `cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml persistence::tests::test_state_path_for_app_mobile` | ❌ Wave 0 |
| PERSIST-01 | `default_state_path()` unchanged for desktop | unit (existing) | `cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml persistence::tests` | ✅ exists |
| PERSIST-02 | `visibilitychange` → `save_state` round-trip | manual-only (on-device) | N/A — requires WKWebView + physical device | manual |
| PERSIST-03 (compat) | v4.0 autosave.json loads without error | unit | `cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml persistence::tests::test_loads_v40_autosave_fixture` | ❌ Wave 0 |
| PERSIST-03 (round-trip) | Background→kill→relaunch restores state | manual-only (on-device) | N/A — requires physical device lifecycle | manual |

**Manual-only justifications:**
- PERSIST-02 and PERSIST-03 (round-trip): WKWebView lifecycle and iOS kill events cannot be
  simulated in host Rust tests. The physical device test (D-54.4a) is the validation gate.

### Sampling Rate
- **Per task commit:** `cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml`
- **Per wave merge:** `just gui-ci`
- **Phase gate:** `just gui-ci` green + on-device round-trip confirmed before `/gsd:verify-work`

### Wave 0 Gaps

- [ ] `hp41-gui/src-tauri/tests/fixtures/v40-autosave.json` — v4.0 era JSON with `version:1` wrapper, `xmem_files` non-empty, `rand_seed` non-zero, `adv_tvm_state` non-null
- [ ] `persistence::tests::test_state_path_for_app_mobile` — unit test asserting the mobile branch resolves to a path containing "Application Support" or "ch.talent-factory.hp41" (requires a mock or conditional compilation trick; alternatively, test the fallback chain directly)
- [ ] `persistence::tests::test_loads_v40_autosave_fixture` — loads the v4.0 fixture via `load_state()`, asserts `xrom_modules == 0b0001_1111`, `is_running == false`, `xmem_files` non-empty

*Note on `test_state_path_for_app_mobile`: Testing the `#[cfg(mobile)]` branch in host tests is
impossible without cross-compiling to the iOS target. The most practical test is:
(a) the compile-time `#[cfg(mobile)]` gate itself (the block only compiles on mobile), and
(b) an integration test that runs on-device and asserts the path contains "Application Support".*

---

## Security Domain

The phase handles local file I/O only — no network calls, no auth, no cryptography.

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | — |
| V3 Session Management | no | — |
| V4 Access Control | no | — |
| V5 Input Validation | yes (partial) | `load_state` returns `Err` on malformed JSON (serde_json); never panics — existing pattern |
| V6 Cryptography | no | — |

### Known Threat Patterns for this stack

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Malformed autosave.json (corrupt/crafted) | Tampering | `load_state` returns `Err` + fallback to fresh state (D-03, existing) |
| Path traversal via `app_local_data_dir` | Spoofing | Tauri's PathResolver does not accept user input; path is system-derived |
| iOS sandbox entitlement violation | Denial of Service | Fallback chain logs error; app starts fresh (not a crash) |

---

## Sources

### Primary (HIGH confidence)
- Codebase: `hp41-gui/src-tauri/src/persistence.rs` — verified all call sites of `default_state_path()`
- Codebase: `hp41-gui/src-tauri/src/lib.rs` — verified auto-save thread structure and `#[cfg(desktop)]` pattern
- Codebase: `hp41-gui/src-tauri/src/commands.rs` — verified `save_state` and `set_pref` use `default_state_path()` / `default_prefs_path()`
- Codebase: `hp41-gui/src-tauri/src/prefs.rs` — verified `default_prefs_path()` pattern mirrors persistence.rs
- Codebase: `hp41-gui/src-tauri/Cargo.toml` — verified `dirs = "6"` already present; no new deps needed
- Codebase: `hp41-gui/src-tauri/Cargo.lock` — verified tauri 2.11.1, dirs 6.0.0
- Source: `dirs-sys-0.5.0/src/lib.rs:39` — verified iOS special-case: `getpwuid_r` disabled on `target_os = "ios"`, `HOME` env var is the only path
- Source: `docs.rs/tauri/2.11.1` — verified `PathResolver::app_local_data_dir(&self) -> Result<PathBuf>`
- Codebase: `hp41-core/tests/fixtures/` — confirmed v20–v33 fixtures exist, v40 does not yet exist
- Codebase: `hp41-core/src/ops/xmem/mod.rs` — confirmed `XmemFile.name` is a file name, not a filesystem path (P-iOS-29 audit pass)
- `.planning/phases/54-ios-persistence-layer/54-CONTEXT.md` — D-54.x decisions locked

### Secondary (MEDIUM confidence)
- Apple Developer Documentation (WebFetch): `HOME` environment variable on iOS is set to the app's sandbox container root
- GitHub tauri-apps/tauri#12552 (WebFetch): confirmed `app_data_dir()` / `app_local_data_dir()` "Permission Denied" bug on iOS; `dirs::home_dir()` workaround confirmed by reporter
- `.planning/research/ARCHITECTURE.md` §309-356 — prescriptive plan for iOS persistence approach A
- `.planning/research/PITFALLS.md` §7 (P-iOS-28 through P-iOS-31) — iOS persistence pitfalls
- `docs/adr/v4.1-002-build-approach.md` — Approach A confirmed; nested workspace #5865 not an issue in Tauri 2.11

### Tertiary (LOW confidence — verify on-device)
- WebSearch results: `visibilitychange` reliability on iOS WKWebView — conflicting signals; standard WKWebView does fire it on Home press in Safari, but Tauri's plumbing on desktop has known issues (#6864, #9524). iOS behavior unconfirmed in this session.
- GitHub tauri-apps/tauri#12276: filesystem paths on iOS don't resolve correctly in all cases — MEDIUM-confidence corroboration of #12552

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all packages verified in Cargo.lock; no new deps
- Architecture: HIGH — derived from codebase inspection; all 4 call sites identified
- iOS path API: MEDIUM — `app_local_data_dir()` return type verified; iOS-specific behavior has one known bug (#12552) with confirmed workaround
- `visibilitychange` reliability: LOW/MEDIUM — architecturally correct for WKWebView; Tauri desktop has known issues; iOS behavior unverified in this session; safety net (30s timer) exists per D-54.2a
- P-iOS-29 path audit: HIGH — verified by codebase inspection; no path strings found in CalcState

**Research date:** 2026-06-02
**Valid until:** 2026-07-02 (30 days; Tauri mobile issues evolve quickly)
