---
phase: 49-onboarding-gui-keyboard-parity
plan: "04"
subsystem: hp41-gui
tags: [react, typescript, tauri, onboarding, keyboard-shortcuts, ipc, state-management]
dependency_graph:
  requires:
    - plan: "49-01"
      provides: "save_state Tauri command, set_pref onboarding_done IPC, GuiPrefs.onboarding_done field"
    - plan: "49-03"
      provides: "OnboardingWizard.tsx component with OnboardingWizardProps interface"
  provides:
    - "App.tsx onboarding state management (onboardingOpen + isFirstRun)"
    - "App.tsx Ctrl+W/R/D/F card reader keyboard shortcuts"
    - "App.tsx F5 / Ctrl+S manual save dispatch with toast feedback"
    - "App.tsx __save_state__ interception before dispatchKeyId"
    - "App.tsx OnboardingWizard render with correct props and mutual exclusion"
    - "SettingsPanel Quick Start section with Show Guide button"
    - "SettingsPanelProps.onShowOnboarding: () => void"
    - "OnboardingWizard.tsx stub (interface contract; full implementation from Plan 49-03)"
  affects:
    - hp41-gui/src/App.tsx
    - hp41-gui/src/SettingsPanel.tsx
    - hp41-gui/src/OnboardingWizard.tsx
tech_stack:
  added: []
  patterns:
    - "Fire-and-forget IPC for non-critical persists (set_pref for onboarding_done)"
    - "Special key ID interception (__save_state__) before dispatchKeyId to avoid backend unknown-key error"
    - "isFirstRun boolean controls Esc dismiss behavior across App and wizard component"
    - "Ctrl+key check FIRST in resolveKeyId to prevent fall-through to letter MAP"
key_files:
  created:
    - hp41-gui/src/OnboardingWizard.tsx
  modified:
    - hp41-gui/src/App.tsx
    - hp41-gui/src/SettingsPanel.tsx
    - hp41-gui/src/SettingsPanel.test.tsx
    - hp41-gui/src/App.test.tsx
decisions:
  - "OnboardingWizard.tsx stub created for TypeScript compilation; Plan 49-03 provides full 5-panel implementation"
  - "isFirstRun=false on re-open via Show Guide allows Esc dismiss; isFirstRun=true on first launch blocks Esc (D-49.9)"
  - "__save_state__ intercepted in handleKey BEFORE dispatchKeyId to satisfy T-49-08 and D-07 invariant"
  - "App.test.tsx beforeEach updated to mock get_prefs returning onboarding_done=true so wizard stays closed during tests"
  - "metaKey (Cmd) included alongside ctrlKey for macOS Ctrl+S/Cmd+S parity (RESEARCH A1)"
requirements_completed:
  - ONBOARD-01
  - ONBOARD-02
  - ONBOARD-05
  - KBD-01
  - KBD-02

metrics:
  duration: "~25 minutes"
  completed: "2026-05-27"
  tasks_completed: 2
  tasks_total: 3
  files_created: 1
  files_modified: 4
---

# Phase 49 Plan 04: App.tsx Wiring + SettingsPanel Quick Start Summary

**App.tsx wired end-to-end: first-run wizard auto-shows via get_prefs check, Ctrl+W/R/D/F dispatch card reader ops, F5/Ctrl+S trigger save with toast, OnboardingWizard rendered with state management and mutual exclusion; SettingsPanel extended with Quick Start / Show Guide button.**

## Performance

- **Duration:** ~25 minutes
- **Started:** 2026-05-27
- **Completed:** 2026-05-27
- **Tasks:** 2 of 3 executed (Task 3 is a human checkpoint — pending visual verification)
- **Files created:** 1 (OnboardingWizard.tsx stub)
- **Files modified:** 4 (App.tsx, SettingsPanel.tsx, SettingsPanel.test.tsx, App.test.tsx)

## Accomplishments

- App.tsx fully wired for onboarding: `onboardingOpen` + `isFirstRun` state, `get_prefs` check auto-opens wizard on first run, `handleOnboardingClose` sets `onboarding_done` via fire-and-forget IPC
- Ctrl+key bindings added FIRST in `resolveKeyId`: Ctrl+W=WPRGM, Ctrl+R=RDPRGM, Ctrl+D=WDTA, Ctrl+F=RDTA, Ctrl+S=save, F5=save (KBD-01 / KBD-02)
- `__save_state__` special key ID intercepted before `dispatchKeyId` with `e.preventDefault()` (T-49-07/T-49-08/T-49-09/T-49-10); shows Saved/Save failed toast
- `onboardingOpen` gate blocks keyboard dispatch while wizard is open; Esc closes wizard in re-open mode only
- Mutual exclusion: ? key and help button close onboarding; `handleShowOnboarding` closes settings first (D-49.9)
- SettingsPanel extended with `onShowOnboarding` prop, Quick Start section, Show Guide button

## Task Commits

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 2 | SettingsPanel Quick Start section + onShowOnboarding prop | fa1133a | SettingsPanel.tsx, SettingsPanel.test.tsx |
| 1 | App.tsx wiring: onboarding, Ctrl+key, save dispatch, OnboardingWizard render | 0414eec | App.tsx, OnboardingWizard.tsx, App.test.tsx |
| 3 | Human checkpoint (visual verification) | PENDING | — |

Note: Task 2 was committed before Task 1 (independent execution order is safe; both are atomic).

## Files Created/Modified

- `hp41-gui/src/App.tsx` — onboarding state variables, extended get_prefs useEffect, handleOnboardingClose, handleShowOnboarding, Ctrl+key bindings in resolveKeyId, __save_state__ interception in handleKey, onboardingOpen gate, Esc handler extension, OnboardingWizard render, SettingsPanel prop threading
- `hp41-gui/src/SettingsPanel.tsx` — onShowOnboarding prop added to SettingsPanelProps, destructured in component; Quick Start section with hr divider and Show Guide button replacing Phase 49 placeholder comment
- `hp41-gui/src/SettingsPanel.test.tsx` — all 6 existing tests updated with `onShowOnboarding={vi.fn()}` prop; 2 new tests: "renders Quick Start section" + "clicking Show Guide calls onClose then onShowOnboarding"
- `hp41-gui/src/App.test.tsx` — beforeEach updated to mock get_prefs returning `{ theme: 'dark', onboarding_done: true }` so wizard stays closed during tests
- `hp41-gui/src/OnboardingWizard.tsx` — new file: stub implementing exact interface (open/onClose/isFirstRun props), Esc handler for re-open mode, placeholder render

## Decisions Made

- `OnboardingWizard.tsx` stub created here for TypeScript compilation; Plan 49-03 creates the full 5-panel component in its own worktree. The orchestrator will merge Plan 49-03's full implementation over this stub.
- `isFirstRun=false` on re-open (via `handleShowOnboarding`) allows Esc to close the wizard. `isFirstRun=true` on first launch blocks Esc (D-49.9 — user must complete wizard or click Skip).
- `metaKey` included alongside `ctrlKey` in resolveKeyId so macOS Cmd+S works equivalently to Ctrl+S (RESEARCH A1 pattern).
- The `__save_state__` special key ID is synthetic — it MUST NOT reach `dispatch_op` or `key_map.rs` which would error (D-07). The interception happens after `resolveKeyId` returns but before `dispatchKeyId` is called.
- App.test.tsx `beforeEach` needed updating: the new `get_prefs` mount effect returns `makeEmptyView()` from the blanket mock, which has `onboarding_done: undefined` → falsy → wizard opens → blocks keyboard dispatch → 5 existing tests fail. Fixed by routing `get_prefs` to return `{ theme: 'dark', onboarding_done: true }`.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed App.test.tsx: get_prefs mock returned makeEmptyView() causing wizard to open in tests**
- **Found during:** Task 1 verification (test run)
- **Issue:** `beforeEach` used `mockInvoke.mockResolvedValue(makeEmptyView())` for all invoke calls. When `get_prefs` returns `makeEmptyView()` (a CalcStateView, not prefs), `prefs.onboarding_done` is `undefined` → `!prefs.onboarding_done` is `true` → `setOnboardingOpen(true)` fires. With `onboardingOpen=true`, the keyboard gate blocks all key dispatch → 5 existing App.test.tsx tests fail (A2, C1, C2, E1, F1).
- **Fix:** Updated `beforeEach` to use `mockInvoke.mockImplementation((cmd) => cmd === 'get_prefs' ? Promise.resolve(DEFAULT_PREFS) : Promise.resolve(makeEmptyView()))` where `DEFAULT_PREFS = { theme: 'dark', onboarding_done: true }`.
- **Files modified:** `hp41-gui/src/App.test.tsx`
- **Verification:** All 192 Vitest tests pass after fix
- **Committed in:** 0414eec (part of Task 1 commit)

---

**Total deviations:** 1 auto-fixed (Rule 1 - Bug)
**Impact on plan:** Required to maintain existing test suite correctness. No scope creep.

## Known Stubs

| Stub | File | Reason |
|------|------|--------|
| Minimal OnboardingWizard render (no 5-panel content) | hp41-gui/src/OnboardingWizard.tsx (lines 41-49) | Plan 49-03 creates the full 5-panel wizard implementation in its own worktree. This stub exports the correct OnboardingWizardProps interface so App.tsx compiles. The orchestrator will merge Plan 49-03's full component over this stub. |

## Checkpoint Pending

**Task 3 (checkpoint:human-verify, gate="blocking")** has NOT been executed per the objective instruction (`autonomous: false`). The checkpoint requires:
- Visual verification of the complete Phase 49 end-to-end experience
- First-run wizard auto-shows (ONBOARD-01)
- Wizard does not reappear after dismissal (ONBOARD-05)
- Show Guide button in settings re-opens wizard (ONBOARD-02)
- Keyboard shortcuts in ? overlay (KBD-03) — requires Plan 49-03
- Expandable function entries (ONBOARD-03) — requires Plan 49-03
- Ctrl+W/R/D/F card reader shortcuts (KBD-01)
- Ctrl+S / F5 save triggers (KBD-02)
- Theme compatibility

The human checkpoint will be triggered by the orchestrator once Plans 49-03 and 49-04 are merged.

## Threat Flags

None — all threat mitigations from the plan's threat model are implemented:
- T-49-07: Ctrl+key check FIRST in resolveKeyId; `return null` for unknown Ctrl combos
- T-49-08: `__save_state__` intercepted before `dispatchKeyId`; never reaches `key_map.rs`
- T-49-09: `e.preventDefault()` on F5 — prevents browser/Tauri WebView reload
- T-49-10: `e.preventDefault()` on Ctrl+S — prevents browser save-page dialog
- T-49-SC: No new packages installed; zero new runtime deps policy maintained

## Self-Check: PASSED

Files created/exist:
- FOUND: hp41-gui/src/App.tsx (modified)
- FOUND: hp41-gui/src/SettingsPanel.tsx (modified)
- FOUND: hp41-gui/src/SettingsPanel.test.tsx (modified)
- FOUND: hp41-gui/src/App.test.tsx (modified)
- FOUND: hp41-gui/src/OnboardingWizard.tsx (created)

Commits exist:
- FOUND: fa1133a (Task 2: SettingsPanel)
- FOUND: 0414eec (Task 1: App.tsx)

Test results: 192 tests pass, 0 failures
TypeScript: tsc --noEmit clean
