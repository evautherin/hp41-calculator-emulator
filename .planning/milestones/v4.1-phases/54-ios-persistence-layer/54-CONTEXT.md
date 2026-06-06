# Phase 54: iOS Persistence Layer - Context

**Gathered:** 2026-06-02
**Status:** Ready for planning

<domain>
## Phase Boundary

Make `CalcState` autosave/restore work inside the iOS app sandbox under Approach A (Tauri v2 Mobile, confirmed in ADR `v4.1-002`), while leaving the desktop `~/.hp41/autosave.json` path completely unaffected. Concretely: resolve the iOS save path from the app container instead of `~/.hp41/`, trigger a save when the app is backgrounded (iOS suspends apps before the 30 s timer can fire), and verify that an existing v4.0 save file still loads (serde backward-compat).

**In scope:** iOS path resolution via `AppHandle::path().app_local_data_dir()` for `autosave.json` **and** `prefs.json`; threading the resolved path through the setup + auto-save thread; a `visibilitychange` background-save trigger in the React frontend; an automated v4.0 backward-compat fixture test; the on-device background→kill→relaunch round-trip. **Covers PERSIST-01, PERSIST-02, PERSIST-03.**

**Out of scope (own phases / deferred):** touch UI / haptics / ALPHA entry / audio (Phase 55); `backgroundThrottlingPolicy` + foreground clock refresh (Phase 56, LIFE-01/02); signing / `PrivacyInfo.xcprivacy` / app icon / `ci-ios.yml` / TestFlight (Phase 57); `cards.rs` / `.raw` / X-MEM file I/O on iOS (deferred to RAW-IOS-01, no iOS file picker this milestone). **No new calculator functions** — `hp41-core` is feature-complete at v4.0; this is a form-factor adapter change only.

</domain>

<decisions>
## Implementation Decisions

### iOS storage directory (PERSIST-01)
- **D-54.1:** The iOS autosave lives in **Application Support**, resolved via `AppHandle::path().app_local_data_dir()` → `Library/Application Support/ch.talent-factory.hp41/autosave.json`. This matches `ARCHITECTURE.md` §"The Correct iOS Paths" and Apple's convention that `Library/Application Support/` holds **app-managed state** while `Documents/` is for **user-authored content**. An RPN autosave is app-managed state, so Documents is rejected — it would also invite the App-Review scrutiny that `PITFALLS.md` P-iOS-30 itself warns about. iCloud-backed by default (restores on device migration); not visible in the Files app (acceptable — no iOS file picker this milestone). **This resolves the ARCHITECTURE.md (Application Support) vs PITFALLS.md P-iOS-30 (Documents) conflict in favor of ARCHITECTURE.md.**
- **D-54.1a:** The desktop path is unchanged: `default_state_path()` (`~/.hp41/autosave.json`) stays exactly as-is for `#[cfg(desktop)]` and existing unit tests. iOS and desktop are **different files on different devices**; the "shared with CLI" property does not apply on iOS (iOS never runs the CLI) — this is correct and expected (`ARCHITECTURE.md` §341).

### Background-save trigger (PERSIST-02)
- **D-54.2:** Add `document.addEventListener("visibilitychange", …)` in the React frontend; when `document.visibilityState === "hidden"`, call `invoke("save_state")` (the `save_state` command already exists from Phase 49 KBD-02). This fires reliably in WKWebView on Home-button / app-switch, needs no Swift plugin, and is the research-recommended Approach-A mitigation (`ARCHITECTURE.md` §"Autosave on Resign-Active").
- **D-54.2a:** **Keep** the existing 30 s auto-save thread on iOS (do not `#[cfg]`-gate it off). It runs in the foreground as cheap crash protection during active use and is harmlessly suspended with the app in the background; the `visibilitychange` event covers the background transition the timer cannot reach on iOS. Belt-and-suspenders, no behavioral change to desktop.
- **D-54.2b:** **No mid-run save guard needed.** Saving while a program is running is harmless because `load_state()` already forces `is_running = false` on load (Pitfall 4). The snapshot-clone-then-save discipline (CR-01: release the Mutex before disk I/O) is preserved.
- **D-54.2c:** `pagehide` and `beforeunload` are explicitly **not** added — `visibilitychange` is the reliable WKWebView signal; the alternatives are largely redundant and risk double-saves.

### Path-migration scope (PERSIST-01 sibling consistency)
- **D-54.3:** Thread `app_local_data_dir()` into **both** `persistence.rs` (`autosave.json`) **and** `prefs.rs` (`prefs.json`). Same one-line `AppHandle` change in both; prevents a latent bug where iOS theme/onboarding selections silently fail to persist (`prefs.rs` currently falls back to `PathBuf::from(".")` when `home_dir()` is `None` on iOS — an undefined relative path). Avoids that papercut surfacing during Phase 55 / TestFlight.
- **D-54.3a:** `cards.rs` (`~/.hp41/cards/`) is **left untouched.** Card / `.raw` / X-MEM file I/O on iOS is deferred (RAW-IOS-01, future milestone; `STACK.md` calls it a non-blocking gap). Card ops are unreachable on iOS without a file picker, so migrating `cards.rs` now would be dead code for a deferred feature.

### PERSIST-03 verification strategy
- **D-54.4:** Prove the "existing desktop save files still load" half with an **automated v4.0 fixture test**: commit a real v4.0-era `autosave.json` fixture and assert `load_state()` deserializes it and `migrate_after_load()` runs. Because the serde path (`StateFile` / `CalcState` / `migrate_after_load`) is byte-for-byte identical across all targets, a passing host/CI test proves iOS compatibility — no manual sandbox injection. Augment the existing `persistence.rs` backward-compat tests with a v4.0 fixture if one is not already present.
- **D-54.4a:** Prove the "survives background→kill→relaunch" half with the **on-device round-trip**: run an RPN session on the physical iPhone, background (Home), kill, relaunch, confirm X/Y/Z/T + program memory + XROM module state are restored. This is the device test that exercises the path + lifecycle wiring end-to-end.
- **D-54.4b:** Manual Xcode "Download Container" file injection is **not** required (rejected as fiddly + non-repeatable). The automated fixture test satisfies P-iOS-31's "backward-compat test before first TestFlight upload" intent in CI.

### Claude's Discretion
- **`app_data_dir` #12552 contingency:** The Tauri bug where `app_data_dir()` (and possibly `app_local_data_dir()`) throws "Permission Denied" on iOS in some configs is delegated to the planner/spike. Attempt `app_local_data_dir()` first (architecturally correct); the documented low-confidence fallback is `dirs::home_dir()` (reportedly returns a container path on iOS). The planner decides the exact fallback wiring after observing real device behavior.
- **`state_path_for_app()` shape:** exact signature/placement of the new `AppHandle`-aware path resolver (`ARCHITECTURE.md` suggests `pub fn state_path_for_app(handle: &tauri::AppHandle) -> PathBuf`), how the resolved `PathBuf` is `move`-captured into the auto-save thread, and the analogous `prefs_path_for_app()` — all implementation detail.
- **`#[serde]` path-string audit (P-iOS-29):** verify no `CalcState` field persists an absolute filesystem path that would break across container-UUID changes; planner audits and notes findings.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Milestone research (drives this phase)
- `.planning/research/ARCHITECTURE.md` §"iOS Persistence and App Lifecycle" (lines ~309–355) — the prescriptive plan: `app_local_data_dir()` path, the 4 concrete `persistence.rs` changes, `visibilitychange` resign-active mitigation, stable-serde state restoration, and the file-touch table (line ~488). **Primary reference.**
- `.planning/research/PITFALLS.md` §7 "iOS Persistence Pitfalls" — P-iOS-28 (`~/.hp41` doesn't exist on iOS, CRITICAL), P-iOS-29 (never persist absolute paths), P-iOS-30 (iCloud backup / Documents-vs-Application-Support — note this doc's Documents recommendation is **overridden by D-54.1**), P-iOS-31 (schema migration on update → v4.0 fixture test).
- `.planning/research/STACK.md` lines ~106/113/343/376 — `tauri-plugin-fs` iOS sandbox path notes, `BaseDirectory` guidance, and the `.raw` iOS file-picker gap that justifies leaving `cards.rs` alone (D-54.3a).
- `.planning/research/SUMMARY.md` — build-approach recommendation + suggested build order.

### Requirements, ADRs & state
- `.planning/REQUIREMENTS.md` — PERSIST-01/02/03 (this phase) + full v4.1 set + RAW-IOS-01 (deferred file I/O).
- `.planning/ROADMAP.md` §"Phase 54" — goal + 4 success criteria.
- `docs/adr/v4.1-002-build-approach.md` — Approach A (Tauri v2 Mobile) confirmed; the reason all React/IPC/`hp41-core` assets carry over unchanged.
- `.planning/phases/53-build-approach-decision-ios-scaffold-spike/53-CONTEXT.md` + `53-04-SUMMARY.md` — D-53.x decisions, `just ios-*` recipes, on-device smoke proof, Frozen-Invariant boundary.

### Code touch points (verified during scout)
- `hp41-gui/src-tauri/src/persistence.rs` — `default_state_path()` (line 32, keep for desktop); add the `AppHandle`-aware resolver; `save_state`/`load_state` (lines 42/56) unchanged; existing backward-compat tests at line ~153 (augment with v4.0 fixture).
- `hp41-gui/src-tauri/src/lib.rs` — `run()` setup (lines 53–116): the autosave load (line 72) + 30 s auto-save thread (lines 100–116) both call the free-standing `default_state_path()` with no `AppHandle`; this is where the iOS path must be threaded in. `#[cfg(mobile)]` / `#[cfg(desktop)]` split lives here.
- `hp41-gui/src-tauri/src/prefs.rs` — `default_prefs_path()` (line 71): same `home_dir().join(".hp41")` pattern; migrate alongside autosave (D-54.3).
- `hp41-gui/src-tauri/src/commands.rs` — `save_state` command (registered in `lib.rs` line 178) is the invoke target for the `visibilitychange` handler.
- `hp41-gui/src/` (React, e.g. `App.tsx`) — where the `visibilitychange` listener + `invoke("save_state")` is added.
- `hp41-gui/src-tauri/src/cards.rs` — `cards_dir()` (line 38): **do NOT modify** (D-54.3a).

### Frozen Invariants in play
- `CLAUDE.md` §"Save-file backward compat" — every `CalcState` field carries `#[serde(default)]`; v1.0–v3.3 (and v4.0) load without migration via `migrate_after_load()`. Two documented serde exceptions (`rand_seed`, `adv_tvm_state`).
- `CLAUDE.md` §"GUI specifics" — "Persistence sharing: CLI + GUI share `~/.hp41/autosave.json`. Auto-save thread releases Mutex BEFORE disk I/O" (CR-01); "No polling (D-11)" — note `visibilitychange`→`save_state` is **event-driven, not polling**, so D-11 is honored.
- `CLAUDE.md` §"Workspace structure" — `hp41-core` never gains filesystem/UI deps; iOS path lives only in the `hp41-gui` adapter.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **`save_state` Tauri command** (Phase 49 KBD-02) — already exists and is registered; the `visibilitychange` handler just invokes it. No new command needed.
- **`load_state()` / `migrate_after_load()`** — platform-agnostic serde path; reused verbatim for the iOS backward-compat fixture test.
- **30 s auto-save thread** (`lib.rs` 100–116) — already implements snapshot-clone-then-save with Mutex released before disk I/O (CR-01); kept on iOS (D-54.2a).
- **`prefs.rs` structure** — mirrors `persistence.rs` path resolution exactly, so the same `AppHandle` threading applies to both (D-54.3).

### Established Patterns
- **Adapter-owns-the-path** — `hp41-core` has no filesystem knowledge; the `hp41-gui` adapter supplies the path. iOS simply supplies a different path (`ARCHITECTURE.md` §25).
- **`#[cfg(desktop)]` / `#[cfg(mobile)]` gating** — already used in `lib.rs` (e.g. `tauri-plugin-autostart` is desktop-only, lines 46–50); the iOS-vs-desktop path branch follows the same idiom.
- **Stable serde wrapper** — `StateFile { version: u32, state: CalcState }` is identical on all platforms (D-06); no format change for iOS.

### Integration Points
- `AppHandle` → path API: the new `state_path_for_app(&handle)` must be called inside `.setup()` where `app.handle()` is available, and the resolved `PathBuf` `move`-captured into the auto-save thread (replacing its own `default_state_path()` call at `lib.rs:104`).
- React `visibilitychange` → `invoke("save_state")` → `commands::save_state` → `persistence::save_state(resolved_path, &snapshot)`.

</code_context>

<specifics>
## Specific Ideas

- The path-resolution conflict between `ARCHITECTURE.md` (Application Support) and `PITFALLS.md` P-iOS-30 (Documents) is **explicitly resolved in favor of Application Support** (D-54.1); the planner should not re-derive this.
- `app_local_data_dir()` is tried first; `dirs::home_dir()` is the only sanctioned fallback if Tauri #12552 ("Permission Denied") bites on-device.
- The v4.0 backward-compat fixture should be a real exported autosave (full XROM module state, `rand_seed`, `adv_tvm_state` present) so the `#[serde(default)]` + two-exception policy is genuinely exercised.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope. (`cards.rs` / `.raw` / X-MEM file I/O on iOS is not a deferred *idea* from this discussion; it is an already-tracked future requirement RAW-IOS-01 with no iOS file picker this milestone.)

</deferred>

---

*Phase: 54-ios-persistence-layer*
*Context gathered: 2026-06-02*
