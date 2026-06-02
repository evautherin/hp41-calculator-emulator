---
phase: 54-ios-persistence-layer
verified: 2026-06-02T20:00:00Z
status: human_needed
score: 4/4
overrides_applied: 0
re_verification: false
human_verification:
  - test: "On-device round-trip: kill + relaunch restores X/Y/Z/T, program memory, and XROM module state on a physical iPhone"
    expected: "Registers, program counter, and all XROM module state (xrom_modules == 0x1F) match exactly the pre-kill values. No fresh-state fallback on relaunch."
    why_human: "54-03-SUMMARY.md records user-approved round-trip on iPhone 15 Pro. On-device behavior cannot be reproduced by host inspection. The SUMMARY serves as the evidence; a host verifier cannot independently re-run a device build."
  - test: "Backgrounding (Home press) fires visibilitychange → save_state before the 30-second timer"
    expected: "No 'background save failed' warning in Xcode console/Console.app; mtime of autosave.json updates within a few seconds of pressing Home (well before the 30s thread would fire). Relaunch after an immediate kill confirms the background save completed."
    why_human: "visibilitychange firing in WKWebView on iOS is a device-runtime observable only. 54-03-SUMMARY records absence of the warning and successful restore after a fast kill. Host inspection cannot replicate this."
  - test: "Save file path on device resolves to Library/Application Support/ch.talent-factory.hp41/autosave.json (not ~/.hp41/)"
    expected: "No 'hp41: app_local_data_dir failed' log in the devicectl console stream. File written to the architecturally-correct Application Support path (Outcome A)."
    why_human: "app_local_data_dir() resolution is device-runtime-only. 54-03-SUMMARY records absence of the fallback log, confirming Tauri #12552 did not manifest on this device/OS. Host cannot replicate."
---

# Phase 54: iOS Persistence Layer — Verification Report

**Phase Goal:** Calculator state is saved and restored correctly inside the iOS app sandbox, with no regression on the desktop path
**Verified:** 2026-06-02T20:00:00Z
**Status:** human_needed (all automated checks VERIFIED; three device-runtime criteria carry forward from human-approved 54-03-SUMMARY.md)
**Re-verification:** No — initial verification

---

## Overall Verdict: PASS-WITH-NOTES

All four Success Criteria are met. Three criteria are device-observable-only and are backed by the user-approved 54-03-SUMMARY.md record. One criterion (SC-4, serde backward-compat) is fully host-verified by a committed fixture test. No gaps or stubs found.

---

## Goal Achievement

### Observable Truths

| # | Truth (Roadmap SC) | Status | Evidence |
|---|---|---|---|
| SC-1 | After kill + relaunch on iPhone, X/Y/Z/T + program memory + XROM state restored exactly | VERIFIED (human-approved) | 54-03-SUMMARY.md Task 2: "X=5678 / Y=1234 + non-trivial program/XROM state restored exactly. User-approved." |
| SC-2 | Backgrounding (Home button) triggers immediate autosave; not reliant on 30s timer alone | VERIFIED (human-approved) | 54-03-SUMMARY.md Task 2: no background-save warning; fast kill (well inside 30s window) + relaunch confirmed the visibilitychange save fired first. visibilitychange useEffect present at App.tsx:943-954 with fire-and-forget invoke. |
| SC-3 | Save file written to iOS sandbox Application Support (not ~/.hp41/); desktop still reads ~/.hp41/autosave.json | VERIFIED (human-approved + codebase) | 54-03-SUMMARY.md: Outcome A — no 'app_local_data_dir failed' log. persistence.rs:47-68: #[cfg(not(mobile))] delegates to default_state_path() (→ ~/.hp41/autosave.json unchanged). lib.rs:64,72,106 all routed through AppHandle-aware resolvers. No default_state_path()/default_prefs_path() calls remain in lib.rs or commands.rs (grep exit 1). |
| SC-4 | v4.0 autosave.json from desktop loads without error in the iOS build; #[serde(default)] policy + two serde-exceptions verified | VERIFIED (host-automatable) | Fixture: tests/fixtures/v40-autosave.json (version:1, xrom_modules:31, rand_seed:"3.141592650", adv_tvm_state present, xmem_files:1). Test: test_loads_v40_autosave_fixture in persistence.rs:278-312 asserts !is_running, xrom_modules==0b0001_1111u8, !xmem_files.is_empty(), rand_seed!=zero, adv_tvm_state.is_some(). |

**Score:** 4/4 truths verified

---

## Required Artifacts

| Artifact | Expected | Status | Details |
|---|---|---|---|
| `hp41-gui/src-tauri/src/persistence.rs` | state_path_for_app(&AppHandle) resolver + v4.0 fixture backward-compat test | VERIFIED | fn state_path_for_app at line 47; #[cfg(mobile)] / #[cfg(not(mobile))] branches; `use tauri::Manager` scoped inside cfg(mobile) block at line 50 (98985e9 build-fix); test_loads_v40_autosave_fixture at line 278 |
| `hp41-gui/src-tauri/src/prefs.rs` | prefs_path_for_app(&AppHandle) resolver | VERIFIED | fn prefs_path_for_app at line 86; mirrors persistence.rs exactly; `use tauri::Manager` scoped inside cfg(mobile) block at line 89 (98985e9 build-fix) |
| `hp41-gui/src-tauri/tests/fixtures/v40-autosave.json` | StateFile-wrapped v4.0 fixture with version:1, xmem_files, both serde-exception fields | VERIFIED | version:1 confirmed; xrom_modules:31 (0b0001_1111); rand_seed:"3.141592650" (non-zero); adv_tvm_state non-null; xmem_files:[{DATFILE}]; xmem_active_file:"DATFILE"; is_running:false |
| `hp41-gui/src/App.tsx` | visibilitychange useEffect invoking save_state | VERIFIED | lines 943-954: useEffect with document.addEventListener/removeEventListener for visibilitychange; guards document.visibilityState==='hidden'; void invoke<void>('save_state').catch(...) fire-and-forget pattern |

---

## Key Link Verification

| From | To | Via | Status | Details |
|---|---|---|---|---|
| `lib.rs setup()` | `persistence::state_path_for_app` | line 72: `persistence::state_path_for_app(app.handle())` | WIRED | Call confirmed at lib.rs:72 |
| `lib.rs setup()` | `prefs::prefs_path_for_app` | line 64: `prefs::prefs_path_for_app(app.handle())` | WIRED | Call confirmed at lib.rs:64 |
| `lib.rs auto-save thread` | `persistence::state_path_for_app` | line 106: resolved BEFORE thread::spawn, PathBuf move-captured | WIRED | `let thread_save_path = persistence::state_path_for_app(&handle)` at lib.rs:106; spawn at lib.rs:107 |
| `commands.rs save_state` | `persistence::state_path_for_app` | `app: AppHandle` param + `persistence::state_path_for_app(&app)` at line 567 | WIRED | Confirmed at commands.rs:561,567 |
| `commands.rs set_pref` | `prefs::prefs_path_for_app` | `app: AppHandle` param + `prefs_path_for_app(&app)` at line 523 | WIRED | Confirmed at commands.rs:494,523 |
| `App.tsx visibilitychange` | `save_state Tauri command` | `document.visibilityState==='hidden'` → `invoke<void>('save_state')` | WIRED | App.tsx:944-949 |
| `#[cfg(not(mobile))] desktop branch` | `default_state_path()` / `default_prefs_path()` | Passthrough to ~/.hp41/ path | WIRED | persistence.rs:64-67, prefs.rs:103-106 |

**No old default_state_path() or default_prefs_path() calls remain in lib.rs or commands.rs.** (grep returned exit 1 — zero matches.)

---

## Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
|---|---|---|---|---|
| `persistence.rs save_state` | `CalcState` snapshot | Clone under Mutex lock from AppState | Yes — live engine state | FLOWING |
| `persistence.rs load_state` | `StateFile` deserialized | serde_json from disk file | Yes — real deserialization + migrate_after_load() | FLOWING |
| `prefs.rs save_prefs` | `GuiPrefs` snapshot | Clone under PrefsState Mutex lock | Yes — live prefs state | FLOWING |
| `App.tsx visibilitychange handler` | No React state — always saves current backend state | invoke('save_state') IPC → commands.rs save_state → CalcState clone from AppState | Yes — backend state at moment of backgrounding | FLOWING |

---

## Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|---|---|---|---|
| Desktop path unchanged | `grep -n ".hp41.*autosave" persistence.rs` | default_state_path() still joins ".hp41" / "autosave.json" at lines 35-36 | PASS |
| No old call sites remain | `grep -c "default_state_path()\|default_prefs_path()" lib.rs commands.rs` | Exit 1 — zero matches | PASS |
| cfg(mobile) present in both resolvers | `grep -n "cfg(mobile)" persistence.rs prefs.rs` | persistence.rs:48 and 64; prefs.rs:87 and 103 | PASS |
| use tauri::Manager in cfg(mobile) blocks | `grep -n "use tauri::Manager" persistence.rs prefs.rs` | persistence.rs:50, prefs.rs:89 | PASS |
| 30s auto-save thread not cfg-gated off | `grep -n "from_secs" lib.rs` | lib.rs:109 `Duration::from_secs(30)` — no cfg gate | PASS |
| visibilitychange count == 2 in App.tsx | `grep -c "visibilitychange" App.tsx` | 2 (addEventListener + removeEventListener) | PASS |
| No pagehide/beforeunload added | `grep -c "pagehide\|beforeunload" App.tsx` | 0 | PASS |
| cards.rs untouched (D-54.3a) | commit stat for 98985e9/054bac3 | cards.rs absent from all phase-54 commit diffs | PASS |
| P-iOS-29 audit: no PathBuf in CalcState | `grep -c "PathBuf\|use std::path" hp41-core/src/state.rs` | Exit 1 — zero matches | PASS |
| v4.0 fixture fields correct | python3 json parse | version:1, xrom_modules:31, rand_seed non-zero, adv_tvm_state present, xmem_files:1, xmem_active_file:"DATFILE", is_running:false | PASS |

---

## Probe Execution

No probe scripts were defined for this phase (device-verification was human-gated per plan 54-03). Step 7c: SKIPPED (no probe-*.sh for this phase; on-device behaviors are human-verified per SUMMARY).

---

## Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|---|---|---|---|---|
| PERSIST-01 | 54-01, 54-03 | iOS save path resolves to app-sandbox container; desktop path unchanged | SATISFIED | state_path_for_app / prefs_path_for_app with cfg(mobile) gate; desktop passthrough confirmed by test; on-device Outcome A (54-03-SUMMARY) |
| PERSIST-02 | 54-02, 54-03 | Backgrounding triggers immediate autosave via visibilitychange | SATISFIED | App.tsx:943-954 visibilitychange → invoke('save_state'); on-device confirm in 54-03-SUMMARY (fast kill + restore) |
| PERSIST-03 | 54-01, 54-03 | State round-trip: kill + relaunch restores complete state; v4.0 backward-compat preserved | SATISFIED | test_loads_v40_autosave_fixture green (host); on-device round-trip user-approved (54-03-SUMMARY) |

---

## Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|---|---|---|---|---|
| — | — | — | — | No anti-patterns found |

No TBD / FIXME / XXX / placeholder patterns found in phase-54-modified files. No empty implementations. No hardcoded empty data passed to rendering paths.

**Notable observation:** The build-fix commit 98985e9 (`use tauri::Manager` inside `#[cfg(mobile)]` blocks) is a genuine 54-01 defect that `just gui-ci` (macOS host build) could not detect because it compiles out the mobile branch. This is recorded as a project learning in 54-03-SUMMARY.md and the fix is verified present in both persistence.rs:50 and prefs.rs:89. The host gate is documented to be blind to `#[cfg(mobile)]` compile errors; `cargo check --target aarch64-apple-ios` is the correct gate for this class of bug.

---

## Human Verification Required

Three of the four success criteria depend on device-runtime observable behavior that no host inspection can reproduce. All three are covered by the user-approved 54-03-SUMMARY.md record.

### 1. Background → Kill → Relaunch Round-Trip (SC-1)

**Test:** Build and install the iOS release IPA on a physical iPhone. Enter a distinctive stack value (e.g. X=5678, Y=1234), set a non-trivial program step, press Home, immediately kill from the app switcher, relaunch.
**Expected:** X=5678, Y=1234 (or whatever pre-kill values were set), program memory, and XROM module state (xrom_modules == 0x1F) are exactly restored. The calculator does not start fresh.
**Why human:** Device I/O — requires physical iPhone, IPA build, and human observation of register values.
**Evidence on record:** 54-03-SUMMARY.md Task 2: "Distinctive stack (X=5678 / Y=1234) + non-trivial program/XROM state set, Home, kill from app switcher, relaunch → X/Y/Z/T + program memory + XROM/assignment state restored exactly. User-approved."

### 2. visibilitychange Fires Before 30s Timer (SC-2)

**Test:** While watching the Xcode console or Console.app stream, press the Home button during a live session. Wait fewer than 30 seconds, then relaunch.
**Expected:** No 'background save failed:' warning appears in the console. State is restored on relaunch, confirming the visibilitychange save fired before the periodic timer could.
**Why human:** WKWebView visibilitychange event firing on iOS Home-press is a device-runtime observable. The JS console.warn output requires physical device + Xcode console access.
**Evidence on record:** 54-03-SUMMARY.md: "no background-save warning surfaced; proven by the successful restore after a fast kill (well inside the 30s timer window)."

### 3. Save Path Inside Sandbox Container (SC-3 iOS half)

**Test:** In the devicectl console stream (or Xcode Organizer), search for 'hp41: app_local_data_dir failed'.
**Expected:** The line is ABSENT, confirming app_local_data_dir() resolved successfully and state lands at Library/Application Support/ch.talent-factory.hp41/autosave.json (Outcome A — #12552 did not manifest).
**Why human:** Tauri PathResolver behavior on device is runtime-only. The fallback log is the only observable signal that the #12552 fallback triggered; its absence confirms the primary path.
**Evidence on record:** 54-03-SUMMARY.md: "No 'hp41: app_local_data_dir failed' line on the console stream → app_local_data_dir() succeeded → Outcome A."

---

## Gaps Summary

No gaps. All must-haves are satisfied:

- All five call sites route through AppHandle-aware resolvers (verified by grep — zero remaining default_* calls in lib.rs/commands.rs).
- The cfg(mobile) / cfg(not(mobile)) gate is correctly applied in both resolver functions.
- The critical build-fix (`use tauri::Manager` in mobile blocks, commit 98985e9) is verified present in both files.
- The desktop path (default_state_path → ~/.hp41/autosave.json) is byte-for-byte unchanged, confirmed by the existing test and by the cfg(not(mobile)) passthrough.
- The v4.0 backward-compat fixture is committed and the test asserts all required fields including the two documented serde-exception fields (rand_seed, adv_tvm_state).
- No CalcState field persists an absolute filesystem path (P-iOS-29 audit confirmed by grep on state.rs — zero PathBuf references).
- No pagehide or beforeunload listeners added (D-54.2c — grep confirmed count 0).
- 30s auto-save thread not cfg-gated off (lib.rs:109 confirmed without any cfg gate).
- cards.rs untouched (D-54.3a — absent from all phase-54 commit diffs).

The `human_needed` status reflects that three of the four success criteria are device-observable-only. The 54-03-SUMMARY.md user-approved record is the authoritative evidence for those criteria; no automated host check can substitute.

---

_Verified: 2026-06-02T20:00:00Z_
_Verifier: Claude (gsd-verifier)_
