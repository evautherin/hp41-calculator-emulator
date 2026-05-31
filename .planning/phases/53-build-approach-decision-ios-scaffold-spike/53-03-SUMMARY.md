---
phase: 53-build-approach-decision-ios-scaffold-spike
plan: 03
subsystem: infra
tags: [ios, tauri-mobile, spike, adr, simulator, wkwebview, build-approach]

# Dependency graph
requires:
  - phase: 53-01
    provides: iOS scaffold (crate-type, just ios-* recipes, gen/apple/)
provides:
  - decisive spike outcome — Approach A (Tauri v2 Mobile) confirmed; #5865 does not manifest in Tauri 2.11
  - ADR docs/adr/v4.1-002-build-approach.md (the build-approach decision record)
  - Simulator-verified touch->engine loop (RPN smoke 2 ENTER 3 + -> 5, SIN; SC-2)
affects: [53-04, 54, 55, 56, 57]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Scriptable Simulator smoke: tauri ios build --target aarch64-sim + xcrun simctl install/launch (avoids the interactive tauri ios dev device picker)"
    - "Spike decision keyed strictly on build FAILURE STAGE (Rust compile vs Xcode assembly vs signing)"

key-files:
  created:
    - "docs/adr/v4.1-002-build-approach.md"
  modified: []

key-decisions:
  - "Approach A (Tauri v2 Mobile) confirmed: just ios-build passed Rust compile + Xcode assembly with NO #5865 nested-workspace path error; stopped only at the expected missing-development-team signing gate (Plans 53-02/53-04). No time-boxed workarounds or workspace-flattening needed (D-53.5/D-53.6)."
  - "Phase 55 stays the CSS/React touch-adaptation path (not a SwiftUI keyboard rewrite, which Approach B would have forced)."

patterns-established:
  - "Simulator smoke via build --target aarch64-sim + simctl install/launch/screenshot — deterministic, non-interactive"

requirements-completed: [BUILD-01, BUILD-02]

# Metrics
duration: ~35 min
completed: 2026-05-31
---

# Phase 53 Plan 03: Build-Approach Spike + ADR Summary

**The decisive `just ios-build` spike passed Rust compile + Xcode assembly with no #5865 nested-workspace path error (stopping only at the expected signing gate), the simulator build launched and ran the RPN smoke through hp41-core, and ADR v4.1-002 records Approach A (Tauri v2 Mobile) as confirmed.**

## Performance

- **Duration:** ~35 min
- **Completed:** 2026-05-31
- **Tasks:** 3
- **Files modified:** 1 created (ADR); spike build artifacts are gitignored/diagnostic

## Accomplishments
- Ran the milestone's gating experiment (`just ios-build`, device triple `aarch64-apple-ios`): Rust compiled, the Xcode assembly step passed (target graph → `GatherProvisioningInputs` → `CreateBuildDescription`), **no #5865 path error**; the only stop was the expected missing development team.
- Built the simulator target (`aarch64-apple-ios-sim`) to `HP-41 Calculator.app`, installed + launched it on the iPhone 17 Pro Simulator (iOS 26.5); the app rendered the React HP-41 UI in WKWebView (screenshot evidence).
- Verified the RPN smoke `2 ENTER 3 + → 5`, then `SIN` dispatched through `hp41-core` and dropped the stack (SC-2 / D-53.8) — full touch→engine loop confirmed on iOS.
- Wrote ADR v4.1-002 recording the observed outcome, cited research, accounted for the Frozen Invariant + the `gen/apple/` gitignore policy, and noted Phases 54–57 are unblocked.

## Task Commits

1. **Task 1: decisive spike (cargo tauri ios build) — A/B oracle** - no commit (diagnostic build; artifacts gitignored; Frozen Invariant verified uncommitted)
2. **Task 2: Simulator RPN smoke (SC-2)** - no commit (human-verified observation)
3. **Task 3: write ADR v4.1-002** - `b062870` (docs)

## Files Created/Modified
- `docs/adr/v4.1-002-build-approach.md` — the build-approach decision record (146 lines)

## Decisions Made
- **Approach A confirmed by failure stage** — the build reached signing (post-assembly), proving the nested-workspace bundler works in 2.11. Approach B (SwiftUI + UniFFI) is not pursued for v4.1; it remains the escalation path had the spike failed at the Xcode-assembly stage.
- **Simulator smoke method** — used the scriptable `build --target aarch64-sim` + `simctl install/launch/screenshot` path because `tauri ios dev` (the `ios-sim` recipe) blocks on an interactive device picker when no device argument is supplied.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug/blocker] `tauri ios dev` interactive device-picker hang**
- **Found during:** Task 2 (Simulator boot via `just ios-sim`)
- **Issue:** `just ios-sim` runs `tauri ios dev` with no device argument; with many simulators available it blocks on an interactive "pick a device" prompt that a non-interactive run cannot answer (the `ios-sim` recipe passes `{{device}}` unquoted, so a space-containing device name would also split).
- **Fix:** Booted the iPhone 17 Pro simulator via `simctl`, built the simulator target with `tauri ios build --target aarch64-sim` (no signing needed), then `xcrun simctl install` + `launch` + `io screenshot`. Deterministic and non-interactive. The `ios-sim` recipe remains valid for the manual hot-reload loop with an explicit `device=` argument.
- **Files modified:** none (method change only; no source/recipe edit)
- **Verification:** app launched (PID), HP-41 UI rendered (screenshot), RPN smoke confirmed by the user.
- **Committed in:** n/a (no file change)

---

**Total deviations:** 1 (method workaround, no code change)
**Impact on plan:** None to deliverables; the spike outcome and SC-2 are fully satisfied. Documented for Phase 55+ (the `ios-sim`/`ios-dev` recipes need an explicit device argument or a quoted/conditional `{{device}}` to run unattended).

## Issues Encountered
- A `just ios-build` / sim build re-emits trivial cosmetic churn in the tracked `gen/apple/project.pbxproj` (bundle-id quote style) and `Info.plist` (trailing newline). Reverted both (not committed); noted in the ADR as the option-(a) tracking trade-off.

## User Setup Required
None for this plan. (App ID registration and device signing are the human-gated Plans 53-02 and 53-04.)

## Next Phase Readiness
- **Build approach is decided (Approach A).** Phases 54–57 can be planned in detail; Phase 55 stays CSS/React.
- Remaining Phase 53 work is human-gated: **53-02** (register App ID — needed before signing) and **53-04** (physical-device install + Xcode signing team).
- The device build (`just ios-build`) will succeed once a development team is selected (53-04) for the registered App ID (53-02).

---
*Phase: 53-build-approach-decision-ios-scaffold-spike*
*Completed: 2026-05-31*
