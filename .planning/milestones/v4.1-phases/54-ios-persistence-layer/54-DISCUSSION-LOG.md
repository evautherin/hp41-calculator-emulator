# Phase 54: iOS Persistence Layer - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-02
**Phase:** 54-ios-persistence-layer
**Areas discussed:** iOS storage directory, Background-save trigger, Path-migration scope, PERSIST-03 verification

---

## iOS storage directory

| Option | Description | Selected |
|--------|-------------|----------|
| Application Support | `app_local_data_dir()` → `Library/Application Support/<bundle>` — app-internal, hidden from Files app, iCloud-backed by default. Matches ARCHITECTURE.md + Apple convention. | ✓ |
| Documents/hp41/ | Per PITFALLS P-iOS-30 — visible in Files app if sharing enabled, iCloud-backed. Risks App-Review scrutiny for non-user-data; no file picker this milestone anyway. | |

**User's choice:** Application Support (Recommended)
**Notes:** Resolves the ARCHITECTURE.md (Application Support) vs PITFALLS.md P-iOS-30 (Documents) conflict in favor of ARCHITECTURE.md. An RPN autosave is app-managed state, not a user-authored document. → D-54.1.

---

## Background-save trigger

| Option | Description | Selected |
|--------|-------------|----------|
| visibilitychange + keep 30s timer | JS `visibilitychange`→`hidden`→`invoke("save_state")`; keep the existing 30s thread (foreground crash protection, harmless when suspended). | ✓ |
| visibilitychange only, drop timer on iOS | Single save path; `#[cfg(not(mobile))]` gate the thread off. Loses foreground crash protection. | |
| Add pagehide for extra coverage | Adds a redundant termination signal; risks double-saves. | |

**User's choice:** visibilitychange + keep 30s timer (Recommended)
**Notes:** No mid-run save guard needed — `load_state()` already forces `is_running=false` (Pitfall 4). → D-54.2 / D-54.2a/b/c.

---

## Path-migration scope

| Option | Description | Selected |
|--------|-------------|----------|
| autosave.json + prefs.json | Thread `app_local_data_dir()` into both; leave `cards.rs` (iOS file I/O deferred). Prevents iOS theme-persistence papercut. | ✓ |
| autosave.json only (strict PERSIST-01) | Tightest boundary; iOS theme won't persist until a later phase. | |
| All three (autosave + prefs + cards) | Migrates `cards.rs` too — dead code for a deferred feature (no iOS file picker). | |

**User's choice:** autosave.json + prefs.json (Recommended)
**Notes:** Same one-line `AppHandle` change in both files; `cards.rs` explicitly left alone per RAW-IOS-01 deferral. → D-54.3 / D-54.3a.

---

## PERSIST-03 verification

| Option | Description | Selected |
|--------|-------------|----------|
| Automated fixture test + device round-trip | Commit a v4.0 `autosave.json` fixture, assert `load_state()`+`migrate_after_load()` (serde is platform-identical → proves iOS in CI); background→kill→relaunch proves path/lifecycle on device. | ✓ |
| Manual on-device file injection | Xcode "Download Container" push of a real desktop save — high fidelity, manual, non-repeatable. | |
| Both: automated + one-time device spot-check | CI test + a bundled-fixture manual confirmation before TestFlight (P-iOS-31). Extra effort for pre-ship assurance. | |

**User's choice:** Automated fixture test + device round-trip (Recommended)
**Notes:** The serde path is byte-for-byte identical across targets, so a passing host/CI fixture test proves iOS load compatibility without manual sandbox injection. → D-54.4 / D-54.4a/b.

---

## Claude's Discretion

- Tauri `app_data_dir` #12552 ("Permission Denied" on iOS) contingency — try `app_local_data_dir()` first, `dirs::home_dir()` fallback only if observed on device; planner decides wiring.
- Exact shape of `state_path_for_app()` / `prefs_path_for_app()` resolvers and how the `PathBuf` is `move`-captured into the auto-save thread.
- `CalcState` absolute-path audit (P-iOS-29) — verify no persisted absolute filesystem paths.

## Deferred Ideas

None — discussion stayed within phase scope. (`cards.rs` / `.raw` / X-MEM iOS file I/O is the already-tracked RAW-IOS-01 future requirement, not a new deferred idea.)
