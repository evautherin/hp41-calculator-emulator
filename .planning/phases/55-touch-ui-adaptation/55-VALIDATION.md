---
phase: 55
slug: touch-ui-adaptation
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-06-03
---

# Phase 55 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.
> Derived from `55-RESEARCH.md` §"Validation Architecture". Touch behaviors that
> require a real Taptic Engine / WKWebView are manual on-device checkpoints (D-55.4);
> everything structurally checkable is automated via Vitest, grep, or iOS `cargo check`.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Vitest 4.1.6 (existing in `hp41-gui`) |
| **Config file** | `hp41-gui/package.json` `test` script |
| **Quick run command** | `cd hp41-gui && npm test` |
| **Full suite command** | `cd hp41-gui && npm test` (no separate full suite) |
| **iOS compile guard** | `cd hp41-gui/src-tauri && cargo check --target aarch64-apple-ios` |
| **Estimated runtime** | ~15 seconds (Vitest) + iOS cargo check |

---

## Sampling Rate

- **After every task commit:** Run `cd hp41-gui && npm test`
- **After any `#[cfg(mobile)]` Rust change:** Run `cargo check --target aarch64-apple-ios` (cfg(mobile) blind-spot guard — invisible to `just gui-ci`)
- **After every plan wave:** Run `npm test` + `just gui-ci`
- **Before `/gsd:verify-work`:** Vitest green + iOS cargo check clean + on-device checkpoints passed
- **Max feedback latency:** ~20 seconds for automated; on-device checkpoints batched at phase end

---

## Per-Task Verification Map

> Task IDs are assigned by the planner; this map binds each requirement to its
> automated check or manual checkpoint so the planner can attach `<automated>` /
> `<manual>` verify blocks to the right tasks.

| Requirement | Behavior | Test Type | Automated Command | Status |
|-------------|----------|-----------|-------------------|--------|
| TOUCH-01 | `.key-touch-target` overlay renders for each of 44 keys | unit | `npm test` (Keyboard render) | ⬜ pending |
| TOUCH-01 | 44pt hit accuracy, no adjacent activation (iPhone SE) | manual | — (D-55.4) | ⬜ pending |
| TOUCH-02 | `viewport-fit=cover` in index.html | grep | `grep 'viewport-fit=cover' hp41-gui/index.html` | ⬜ pending |
| TOUCH-02 | Portrait lock in project.yml AND generated Info.plist | grep | `grep -L LandscapeLeft gen/apple/project.yml gen/apple/hp41-gui_iOS/Info.plist` | ⬜ pending |
| TOUCH-03 | `.key-touch-target` has `touch-action: manipulation` | grep | `grep 'touch-action: manipulation' hp41-gui/src/App.css` | ⬜ pending |
| TOUCH-03 | No tap delay / no highlight flash | manual | — (D-55.4) | ⬜ pending |
| TOUCH-04 | AlphaTouchInput renders when `alpha` active + `isIos` | unit | `npm test` (mock isIos, annunciators.alpha) | ⬜ pending |
| TOUCH-04 | AlphaTouchInput renders when `modal_requires_alpha_label` + `isIos` (D-55.2) | unit | `npm test` (mock isIos, modal_requires_alpha_label) | ⬜ pending |
| TOUCH-04 | Input field font-size is 16px (iOS zoom prevention) | grep | `grep 'font-size: 16px' hp41-gui/src/App.css` | ⬜ pending |
| TOUCH-04 | Bar stays above iOS keyboard | manual | — (D-55.4) | ⬜ pending |
| TOUCH-05 | Haptic tier classification (pure `getHapticTier`) | unit | `npm test` | ⬜ pending |
| TOUCH-05 | Haptics compile on iOS target | cargo | `cargo check --target aarch64-apple-ios` | ⬜ pending |
| TOUCH-05 | Haptic actually fires on key press | manual | — (D-55.4) | ⬜ pending |
| TOUCH-06 | Audio audible on first tap after launch | manual | — (D-55.4) | ⬜ pending |
| TOUCH-07 | Annunciators + stack legible at SE scale | manual | — (D-55.4) | ⬜ pending |
| TOUCH-08 | Distinct haptic tier per key type (SHIFT heavier than digit) | manual | — (D-55.4) | ⬜ pending |
| TOUCH-08 | Error haptic fires on DATA ERROR / NO ROOM | manual | — (D-55.4) | ⬜ pending |
| TOUCH-08 | Error haptic does not double-fire (`errorHapticFiredRef` guard) | unit | `npm test` | ⬜ pending |
| TOUCH-09 | Bottom sheets render on `isIos`, absent on desktop | unit | `npm test` (both isIos branches) | ⬜ pending |
| TOUCH-10 | Stack panel collapses on iOS (chevron toggle) | unit | `npm test` (mock isIos, click chevron) | ⬜ pending |
| TOUCH-11 | `overscroll-behavior: none` retained in index.css | grep | `grep 'overscroll-behavior: none' hp41-gui/src/index.css` | ⬜ pending |
| TOUCH-11 | Rubber-band overscroll absent on device | manual | — (D-55.4) | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `hp41-gui/src/AlphaTouchInput.test.tsx` — TOUCH-04 (renders on both triggers: `alpha` and `modal_requires_alpha_label`; routes chars correctly; absent on desktop)
- [ ] `hp41-gui/src/BottomSheet.test.tsx` — TOUCH-09 (renders on iOS, absent on desktop, toggle expand/collapse)
- [ ] Haptic tier classification unit test (new file or extend `Keyboard.test.tsx`) — TOUCH-05 (`getHapticTier` pure function: shift→Heavy, enter/sto/rcl/xeq/gto/r_s/rtn/sst/bst→Medium, else→Light)
- [ ] Error-haptic double-fire guard test — TOUCH-08 (`errorHapticFiredRef` resets only when error clears)

*Existing Vitest infrastructure covers all other automatable requirements via grep/cargo-check.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| 44pt hit accuracy, no adjacent activation | TOUCH-01 | Simulator uses mouse — no fat-finger test | On iPhone SE, tap every one of the 44 keys; confirm none activates a neighbor |
| No perceptible tap delay / no highlight flash | TOUCH-03 | 350ms delay + highlight not reproduced in Simulator | First tap + rapid successive taps feel instant; no grey flash |
| AlphaTouchInput stays above keyboard | TOUCH-04 | Simulator keyboard inset differs (Tauri #10631) | Enter ALPHA + a label modal; bar visible above software keyboard |
| Haptics felt, correct tier, error pattern | TOUCH-05/08 | No Taptic Engine in Simulator | Digit=light, SHIFT=heavy, ENTER=medium; SQRT(-1)→error double-tap |
| Audio audible on first use | TOUCH-06 | AudioContext suspend behavior differs in Simulator | TONE 9 audible on first tap after cold launch |
| Annunciator/stack legibility | TOUCH-07 | Real-screen DPI/contrast | Read SHIFT/ALPHA annunciators + X/Y/Z/T without zoom |
| Rubber-band overscroll absent | TOUCH-11 | Simulator approximates bounce | Drag calculator body — no outer bounce; sheet inner-scrolls |

Layout / safe-area / overscroll may be **smoke-checked** in the Simulator first; the **authoritative** pass is on iPhone SE (D-55.4).

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies or a documented manual checkpoint
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify (manual checkpoints batched separately)
- [ ] Wave 0 covers all MISSING references (4 test files above)
- [ ] No watch-mode flags
- [ ] iOS `cargo check` task present after any `#[cfg(mobile)]` change
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
