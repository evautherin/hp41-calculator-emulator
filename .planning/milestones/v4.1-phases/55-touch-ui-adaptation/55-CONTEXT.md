# Phase 55: Touch UI Adaptation - Context

**Gathered:** 2026-06-03
**Status:** Ready for planning

<domain>
## Phase Boundary

Make the HP-41 calculator **fully operable by touch on an iPhone**: all 44 keys
reachable at ≥44×44pt, a portrait-locked layout that respects safe-area insets,
immediate press feedback with the 350ms tap-delay and tap-highlight flash gone,
ALPHA + label/name touch text entry, per-key haptics (with intensity tiers and a
distinct error haptic), audio (TONE/BEEP) unlock on first gesture, legible
annunciators/stack, pull-up bottom sheets for print + PRGM panels, a collapsible
stack panel, and suppressed rubber-band overscroll. Requirements TOUCH-01..11.

**Form-factor only.** The engine is feature-complete at v4.0 — no new calculator
functions, no `hp41-core` changes. Every change is in `hp41-gui` (React + CSS +
one new Tauri command + one Tauri plugin) and must be **iOS-gated so the desktop
and macOS menu-bar apps are not regressed**.

**Design is already locked by `55-UI-SPEC.md` (approved design contract).** This
discussion did NOT re-open anything the UI-SPEC settled (hit-target overlay
architecture, safe-area method, haptic tiers, audio-resume pattern, bottom-sheet
+ collapsible-stack design, typography/color/spacing, portrait lock, ALPHA
input-vs-grid → input wins). It captured only the architecture/process decisions
the contract left open, plus one scope correction (modal name/label touch entry).

**Out of scope (own phases):** app-lifecycle background-throttling + clock-on-resume
(Phase 56); signing / `PrivacyInfo.xcprivacy` / app icon / `ci-ios.yml` /
TestFlight (Phase 57). Landscape, iPad, on-screen ALPHA character grid → v4.2.

</domain>

<decisions>
## Implementation Decisions

### iOS platform detection (was an open gray area in UI-SPEC)
- **D-55.1:** Gate all iOS-only behaviors behind a **new `is_ios` Tauri command**
  that mirrors the existing `is_macos` command (`hp41-gui/src-tauri/src/commands.rs:544`,
  consumed in `App.tsx:495`). Authoritative via compile-time `cfg(target_os = "ios")`,
  testable, and consistent with the established precedent. **Reject** the
  JS-only `window.__TAURI_INTERNALS__?.platform` check the UI-SPEC floated as an
  alternative. Requires a permission TOML in `permissions/is-ios.toml` + a
  `capabilities/default.json` entry (run `cargo check` first to generate the
  permission registry — per CLAUDE.md Tauri v2.11 rule). One frontend consumer
  sets an `isIos` flag (same shape as the existing `isMacos` state) that gates
  the bottom sheets, collapsible stack, ALPHA bar, and touch text entry.

### Touch text-entry scope (scope correction vs UI-SPEC)
- **D-55.2:** Touch text entry must cover **BOTH** (a) ALPHA-register mode
  (`annunciators.alpha === true`, the only case the UI-SPEC's `AlphaTouchInput`
  handles) **AND** (b) the modal name/label prompts. The desktop GUI already
  types letters into name-entry modals via `modal_requires_alpha_label`,
  single-printable-key → `alpha_<X>` routing, and `submit_modal_with_label`
  (see `App.tsx` ~lines 60, 102, 149-156, 589, 630-660). On an iPhone there is
  **no hardware keyboard**, so without (b) the FUNCTION NAME? / LBL / XEQ / GTO /
  CLP / ASN-label prompts would be impossible to complete by touch — the calc
  would not be "fully operable by touch" (the phase goal). The `AlphaTouchInput`
  component's trigger condition therefore expands from `annunciators.alpha` to
  also fire when `modal_requires_alpha_label === true`, routing its characters
  through the existing modal-label dispatch path (`submit_modal_with_label` /
  `alpha_<X>`) rather than only the ALPHA-register path.
- **D-55.3:** The on-screen ALPHA **character grid** remains deferred to v4.2
  (UI-SPEC decision unchanged) — the native iOS keyboard `<input type="text">`
  approach is the chosen mechanism for both (a) and (b).

### On-device verification strategy
- **D-55.4:** Verify with **inline human checkpoints** (the proven Phase 53/54
  pattern): the executor automates all code, then pauses at explicit checkpoint
  tasks with click-by-click on-device steps for the criteria that are only
  observable on real hardware — 44pt hit-target accuracy on iPhone **SE** (the
  smallest target, no accidental adjacent activation), haptics actually felt with
  the correct tier per key type, error haptic on DATA ERROR / NO ROOM, TONE/BEEP
  audible on first use after launch, no perceptible tap delay / no highlight
  flash, and the bottom-sheet / collapsible-stack / overscroll behaviors. Layout,
  safe-area, and overscroll can be smoke-checked in the Simulator first, but the
  authoritative pass is on device.

### Plan decomposition
- **D-55.5:** **No decomposition preference from the user — `gsd-planner` chooses
  plan boundaries and wave parallelization.** (For reference only, not a
  constraint: a natural staging is hit-targets+safe-area+tap-feedback →
  haptics+audio → ALPHA/modal text entry → bottom sheets + collapsible stack +
  overscroll. The planner is free to use, reshape, or ignore this.)

### Claude's Discretion
- **D-55.5** explicitly delegated plan/wave shape to the planner.
- Exact permission-file naming, the `isIos` state plumbing shape, and how the
  `AlphaTouchInput` trigger is refactored to cover both ALPHA-register and
  modal-label cases (D-55.2) are left to research/planning, provided they honor
  the UI-SPEC's visual contract and the iOS-gating / no-desktop-regression rule.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Design contract (locks the entire visual/interaction layer — read FIRST)
- `.planning/phases/55-touch-ui-adaptation/55-UI-SPEC.md` — approved design
  contract. Locks hit-target overlay architecture (`.key-touch-target`),
  safe-area insets (`env(safe-area-inset-*)` + `viewport-fit=cover`), tap-feedback
  contract (`touch-action: manipulation`, `pointerdown`/`pointerup`,
  `-webkit-tap-highlight-color: transparent`), haptic tiers
  (Light/Medium/Heavy/Error via `tauri-plugin-haptics` 2.3.2), audio-resume
  (`ensureAudioResumed`), `AlphaTouchInput` contract, bottom-sheet + collapsible-
  stack + annunciator contracts, all CSS classes, typography/color/spacing
  tokens, portrait-lock requirement, and the frozen-invariant respect list.

### Requirements & state
- `.planning/REQUIREMENTS.md` §"Touch" — TOUCH-01..11 (this phase's requirements).
- `.planning/STATE.md` §"Accumulated Context" — pre-resolved iOS decisions
  (44pt targets, haptics plugin, audio resume, ALPHA option-A, background
  throttling, bundle ID) + the iOS pitfall table (P-iOS-05 audio-suspended,
  P-iOS-06 safe-area/keyboard-overlap/flicker, P-iOS-20 desktop key targets,
  P-iOS-23 ALPHA-no-keyboard).
- `.planning/ROADMAP.md` §"Phase 55" — goal + 6 success criteria.

### Milestone research (touch / HIG context)
- `.planning/research/PITFALLS.md` — full iOS pitfall catalog (P-iOS-05/06/20/23
  are the Phase-55-relevant ones).
- `.planning/research/FEATURES.md` — touch / HIG feature context.
- `.planning/research/ARCHITECTURE.md` / `.planning/research/STACK.md` — Approach A
  (Tauri v2 Mobile) integration + tool/version matrix.

### Prior-phase decisions carried forward
- `.planning/phases/53-build-approach-decision-ios-scaffold-spike/53-CONTEXT.md` —
  Approach A confirmed; `just ios-*` recipes; iOS 14.0 target; `gen/apple/` build.
- `.planning/phases/54-ios-persistence-layer/54-CONTEXT.md` — iOS sandbox paths;
  `visibilitychange` background-save; on-device install caveat.
- `docs/adr/v4.1-002-build-approach.md` — recorded build approach (Approach A).

### Code touch points (verified during scout)
- `hp41-gui/src-tauri/src/commands.rs:544` — `is_macos()` command (precedent for
  the new `is_ios()` per D-55.1).
- `hp41-gui/src/App.tsx:495` — `invoke('is_macos')` → `setIsMacos` (precedent for
  the `isIos` consumer).
- `hp41-gui/src/App.tsx` ~60 (`modal_requires_alpha_label`), ~102-103
  (`submit_modal_with_label` magic-prefix), ~149-156 (single-key → `alpha_<X>`),
  ~589 / ~630-660 (alphaChar routing into LBL/XEQ/GTO/CLP/ASN-label modals) —
  the existing typed-text-into-modal path that D-55.2 must make touch-capable.
- `hp41-gui/src/key_map.ts` — string-id → Op resolver (`key_map::resolve`).
- `hp41-gui/src-tauri/Cargo.toml` + `hp41-gui/package.json` — add
  `tauri-plugin-haptics` 2.3.2 (per UI-SPEC; not yet present).
- `hp41-gui/src-tauri/permissions/` + `capabilities/default.json` — new TOML for
  `is_ios` (and haptics plugin capability) per CLAUDE.md Tauri v2.11 rule.
- `gen/apple/project.yml` — restrict `UISupportedInterfaceOrientations` to
  portrait only (per UI-SPEC orientation-lock action).
- `hp41-gui/src/App.css`, `index.css`, `scale.ts` (DESIGN 392×900), `main.tsx`
  (ScaledApp/computeScale) — CSS + scaling touch points listed in the UI-SPEC.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **`is_macos` command + `isMacos` state** — exact template to clone for `is_ios`
  / `isIos` (D-55.1). Same Rust `#[tauri::command]` + `cfg(target_os)` shape, same
  one-shot `invoke().then(setX)` consumer in `App.tsx`.
- **Existing modal-label text path** — `modal_requires_alpha_label`,
  `alpha_<X>` routing, and `submit_modal_with_label` already work on desktop;
  D-55.2 reuses this dispatch path for touch rather than inventing a new one.
- **Existing `.key-pressed` + `scale(0.92)` + 80ms transition** — press-feedback
  visual already implemented in the SVG; Phase 55 only swaps the trigger to
  `onPointerDown`/`onPointerUp` and gates the overlay/haptic on iOS.
- **`computeScale` + `ResizeObserver`** — already recomputes on content-size
  change; safe-area padding shrinks the content node and is picked up for free
  (UI-SPEC: no `scale.ts` change needed).

### Established Patterns
- **iOS-gating discipline** — every touch change is wrapped behind `isIos` so
  desktop + macOS menu-bar behavior is byte-for-byte unchanged (mirrors the
  `#[cfg(target_os = "macos")]` discipline already in the codebase).
- **Tauri v2.11 permission flow** — new command → TOML in `permissions/` →
  reference in `capabilities/default.json` → `cargo check` to generate registry.
- **`just` is the sole task runner** — iOS build/verify via the `just ios-*`
  recipes formalized in Phase 53.
- **Inline human-checkpoint execution** (Phase 53/54) — the verification model
  for D-55.4.

### Integration Points
- Touch → `.key-touch-target` overlay `onPointerDown` (haptic + press visual +
  `ensureAudioResumed`) → `onClick`/`onPointerUp` → `key_map.resolve` →
  `invoke('dispatch_op')` → `CalcStateView` → re-render (identical data flow to
  desktop; only the event source/overlay/haptic are new).
- Error haptic hooks the post-IPC `CalcStateView` inspection (`display_str` /
  event buffer for `DATA ERROR` / `NO ROOM`), guarded against double-fire.
- `AlphaTouchInput` fixed bar tracks the iOS keyboard via
  `window.visualViewport` resize; z-index 80 (above help/wizard/settings 60-70).

</code_context>

<specifics>
## Specific Ideas

- iOS detection should be a real Rust command (`is_ios`) — not a JS internals
  sniff — specifically because `is_macos` already set that precedent and the user
  wants consistency/authority over convenience.
- "Fully operable by touch" is taken literally: entering a program **label/name**
  by touch is in scope, not just the ALPHA register (D-55.2). This is the one
  place Phase 55 goes *beyond* the literal UI-SPEC text.
- Verify on iPhone **SE** specifically (smallest device) for hit-target accuracy.
- Audio silent-switch muting is accepted as HP-41-faithful and documented in
  Phase 57 release notes, not the in-app UI (per UI-SPEC).

</specifics>

<deferred>
## Deferred Ideas

- On-screen ALPHA **character grid** (alternative to the iOS keyboard) — v4.2
  enhancement if user research demands it (UI-SPEC + D-55.3).
- Annunciator size bump to 13px — only if on-device legibility testing reveals an
  issue; explicitly **non-blocking** for Phase 55 (UI-SPEC).
- Bottom-sheet **swipe** gesture (tap-to-toggle ships in Phase 55) — v4.2.
- Landscape orientation, iPad universal layout, Android — v4.2+ (STATE.md
  Deferred Items).

</deferred>

---

*Phase: 55-touch-ui-adaptation*
*Context gathered: 2026-06-03*
