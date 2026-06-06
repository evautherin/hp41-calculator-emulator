---
phase: 53-build-approach-decision-ios-scaffold-spike
plan: 04
subsystem: infra
tags: [ios, signing, device-install, devicectl, p-ios-09, xcode-env, developer-mode]

# Dependency graph
requires:
  - phase: 53-02
    provides: registered App ID ch.talent-factory.hp41 (signing unblocked)
  - phase: 53-03
    provides: confirmed Approach A + buildable scaffold
provides:
  - signed debug/release iOS build installed + launched on a physical iPhone 15 Pro (BUILD-03/SC-3)
  - on-device RPN smoke verified (2 ENTER 3 + -> 5, SIN -> 0.0872 = sin 5°) through hp41-core
  - P-iOS-09 fix (build-phase PATH via .xcode.env/.xcode.env.local) — reusable by Phases 54-57 + CI
  - persistent signing config in project.yml (DEVELOPMENT_TEAM 2P4R8QSWT4, automatic)
affects: [54, 55, 56, 57]

# Tech tracking
tech-stack:
  added:
    - ".xcode.env (committed, portable PATH) + .xcode.env.local (gitignored, machine PATH)"
  patterns:
    - "Device install via release `just ios-build` (IPA) + `xcrun devicectl install/launch` — bypasses the Xcode-Run dev-server-addr panic"
    - "Signing baked into project.yml so it survives xcodegen/tauri-ios-init regeneration"

key-files:
  created:
    - "hp41-gui/src-tauri/gen/apple/.xcode.env (committed)"
    - "hp41-gui/src-tauri/gen/apple/.xcode.env.local (gitignored)"
  modified:
    - "hp41-gui/src-tauri/gen/apple/project.yml (build-script env sourcing + DEVELOPMENT_TEAM)"
    - "hp41-gui/src-tauri/gen/apple/hp41-gui.xcodeproj/project.pbxproj (regenerated)"
    - "hp41-gui/src-tauri/gen/apple/hp41-gui.xcodeproj/.../hp41-gui_iOS.xcscheme (regenerated)"
    - "hp41-gui/src-tauri/.gitignore (ignore .xcode.env.local)"

key-decisions:
  - "Device install uses the release `just ios-build` IPA + `xcrun devicectl` rather than Xcode ⌘R, which panics on the missing tauri dev-server addr file (debug ⌘R expects `tauri ios dev` running)."
  - "DEVELOPMENT_TEAM 2P4R8QSWT4 (Talent Factory AG) + automatic signing committed in project.yml — team IDs are not secret and this keeps the device build self-signing across regeneration."

patterns-established:
  - "P-iOS-09 build-phase PATH: build script sources .xcode.env (portable, $HOME/.cargo/bin) + .xcode.env.local (machine, nvm node) before npm"

requirements-completed: [BUILD-03]

# Metrics
duration: ~45 min (incl. P-iOS-09 + signing + Developer Mode debugging)
completed: 2026-06-02
---

# Phase 53 Plan 04: On-Device Install Summary

**A signed iOS build installs and launches on a physical iPhone 15 Pro, and the RPN smoke 2 ENTER 3 + → 5 then SIN → 0.0872 (sin 5°) dispatches through hp41-core on real hardware — after fixing the Xcode build-phase PATH (P-iOS-09) and configuring automatic signing.**

## Performance

- **Duration:** ~45 min
- **Completed:** 2026-06-02
- **Tasks:** 2 (both human-gated checkpoints) + 2 auto-fixed deviations
- **Files modified:** 4 + 1 created (committed) + 1 gitignored

## Accomplishments
- Selected the signing team (Talent Factory AG / `2P4R8QSWT4`); Xcode resolved a development provisioning profile + "Apple Development: Daniel Senften" certificate for the registered App ID (no signing error).
- Produced a **signed `HP-41 Calculator.ipa`** via `just ios-build` (full pipeline: Rust → Xcode assembly → automatic signing → IPA export).
- Installed + launched it on a physical **iPhone 15 Pro** via `xcrun devicectl` (after enabling Developer Mode).
- Verified the on-device RPN smoke `2 ENTER 3 + → 5`, then `SIN → 0.0872` (sin 5° in DEG) — engine dispatch through `hp41-core` confirmed on hardware (**BUILD-03 / SC-3**).

## Task Commits

1. **Task 1: trust device + select signing team (human-action)** - covered by `5381304` (persistent signing config) + the human Xcode action
2. **Task 2: install on device + RPN smoke (human-verify)** - no code commit (build artifacts gitignored; on-device observation)

**Deviation fix commit:** `5381304` (P-iOS-09 PATH + signing)

## Files Created/Modified
- `gen/apple/.xcode.env` (committed) / `.xcode.env.local` (gitignored) — build-phase PATH (P-iOS-09)
- `gen/apple/project.yml` — build script sources the env files; `DEVELOPMENT_TEAM` + `CODE_SIGN_STYLE: Automatic`
- `gen/apple/hp41-gui.xcodeproj/project.pbxproj` + `hp41-gui_iOS.xcscheme` — regenerated via `xcodegen`
- `hp41-gui/src-tauri/.gitignore` — ignore `.xcode.env.local`

## Decisions Made
- **Device-install flow:** release `just ios-build` IPA + `devicectl` (not Xcode ⌘R) — see deviation #2.
- **Committed signing:** `DEVELOPMENT_TEAM 2P4R8QSWT4` in `project.yml` (org-private repo; team IDs aren't secret; survives regeneration).

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] P-iOS-09 — Xcode build phase could not find cargo/node**
- **Found during:** Task 2 (first device build) — `Command PhaseScriptExecution failed` in the "Build Rust Code" phase.
- **Issue:** Xcode (GUI-launched) does not inherit the interactive shell PATH; `cargo` (`~/.cargo/bin`) and `node`/`npm` (via nvm) were absent, so the build script aborted.
- **Fix:** Made the build script source `$SRCROOT/.xcode.env` + `.xcode.env.local` before `npm`; added a committed portable `.xcode.env` (`$HOME/.cargo/bin`) and a gitignored machine-local `.xcode.env.local` (nvm node bin); regenerated the pbxproj from `project.yml` via `xcodegen`. Verified under a stripped PATH that cargo/npm/node resolve.
- **Files modified:** project.yml, .xcode.env(.local), pbxproj, .gitignore
- **Verification:** `just ios-build` produced a signed IPA; `xcodebuild` under a stripped PATH ran the script phase past the PATH error.
- **Committed in:** `5381304`

**2. [Rule 3 - Blocking] Xcode debug ⌘R panics on the missing Tauri dev-server addr file**
- **Found during:** Task 2 — after the PATH fix, the script reached `tauri ios xcode-script`, which panicked: `failed to read missing addr file ch.talent-factory.hp41-server-addr`.
- **Issue:** A direct Xcode **debug** Run expects a running `tauri ios dev` (it injects the dev-server URL into WKWebView). The plan assumed Xcode Run would install directly.
- **Fix:** Switched the device-install flow to a **release `just ios-build`** (bundles the frontend — no dev server) → signed IPA → `xcrun devicectl device install/launch`. Documented in the ADR/SUMMARY for Phases 54-57.
- **Files modified:** none (flow change)
- **Verification:** signed IPA installed + launched on the iPhone; on-device smoke passed.
- **Committed in:** n/a (method)

---

**Total deviations:** 2 auto-fixed (both Rule 3 blocking) + 1 environment action (Developer Mode).
**Impact on plan:** Necessary to achieve the on-device install (BUILD-03/SC-3). The P-iOS-09 fix and signing config are durable improvements that unblock all later iOS phases.

## Issues Encountered
- **Developer Mode disabled** on the iPhone (iOS 16+) blocked `devicectl install` (CoreDeviceError 10005). Resolved by the user enabling Settings → Privacy & Security → Developer Mode (+ restart).
- `xcodegen` (homebrew) regenerated the scheme with minor version drift (and corrected the product name to `HP-41 Calculator.app`); functionally verified by the successful signed device build.

## User Setup Required
**Completed.** See [53-USER-SETUP.md](./53-USER-SETUP.md) — device trusted + Developer Mode on, signing team selected. No distribution secrets created (Phase 57).

## Next Phase Readiness
- **Phase 53 is complete.** All four success criteria met (SC-1 scaffold, SC-2 Simulator smoke, SC-3 device smoke, SC-4 approach decided); BUILD-01..04 done.
- iOS build pipeline is end-to-end working (scaffold → simulator → signed device install). Phases 54–57 can be planned in detail (Approach A).
- Carry-forward for Phase 57 CI: the `.xcode.env` PATH pattern and `DEVELOPMENT_TEAM` config; the Xcode-debug-⌘R dev-server caveat (use `tauri ios build` or `tauri ios dev`).

---
*Phase: 53-build-approach-decision-ios-scaffold-spike*
*Completed: 2026-06-02*
