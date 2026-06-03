# Phase 55: Touch UI Adaptation - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-03
**Phase:** 55-touch-ui-adaptation
**Areas discussed:** iOS platform detection, Touch text-entry scope, On-device verification, Plan decomposition

> Note: `55-UI-SPEC.md` (approved design contract) already locked the entire
> visual/interaction layer. This discussion deliberately covered ONLY the
> architecture/process decisions the contract left open, plus one scope
> correction. No UI-SPEC decision was re-opened.

---

## iOS platform detection

| Option | Description | Selected |
|--------|-------------|----------|
| Add `is_ios` command | Mirror existing `is_macos` (commands.rs:544). Rust `cfg(target_os)`, testable, authoritative. Needs permission TOML + capability entry. | ✓ |
| JS-only `__TAURI_INTERNALS__` | `window.__TAURI_INTERNALS__?.platform === 'ios'` in frontend. No Rust/permission change but relies on internal field, diverges from precedent. | |

**User's choice:** Add `is_ios` command
**Notes:** Consistency with the existing `is_macos` pattern and compile-time authority preferred over the convenience of a JS internals sniff. → D-55.1

---

## Touch text-entry scope

| Option | Description | Selected |
|--------|-------------|----------|
| ALPHA + modal prompts | Touch text entry for BOTH ALPHA-register mode AND the `modal_requires_alpha_label` name/label prompts (LBL/XEQ/GTO/CLP/ASN). Required for "fully operable by touch". | ✓ |
| ALPHA register only | Match UI-SPEC literally; defer name/label-by-touch. Risks leaving program entry unusable by touch. | |

**User's choice:** ALPHA + modal prompts
**Notes:** The desktop GUI already types letters into name-entry modals via the existing `submit_modal_with_label` / `alpha_<X>` path; on iPhone there is no hardware keyboard, so this path must also be touch-capable. This is the one place Phase 55 extends beyond the literal UI-SPEC text. → D-55.2

---

## On-device verification

| Option | Description | Selected |
|--------|-------------|----------|
| Inline device checkpoints | Reuse Phase 53/54 pattern: executor automates code, then pauses at explicit human checkpoints with click-by-click on-device verification (haptics, audio, tap feel, SE hit-target accuracy). | ✓ |
| Simulator + light manual | Auto-verify layout/safe-area/overscroll in Simulator; one consolidated manual device pass at the end. Faster, less rigorous per-criterion. | |

**User's choice:** Inline device checkpoints
**Notes:** Most success criteria are only observable on real hardware (iPhone SE for hit-target accuracy). → D-55.4

---

## Plan decomposition

| Option | Description | Selected |
|--------|-------------|----------|
| Staged for device validation | Split into validatable waves (targets+safe-area+feedback → haptics+audio → ALPHA/text → sheets+stack+overscroll). | |
| Let the planner decide | No decomposition preference; hand the full requirement set to gsd-planner. | ✓ |

**User's choice:** Let the planner decide
**Notes:** Plan boundaries and wave parallelization delegated to gsd-planner. A natural staging is noted in CONTEXT.md for reference only, not as a constraint. → D-55.5

---

## Claude's Discretion

- Plan/wave shape delegated to the planner (D-55.5).
- Permission-file naming, `isIos` state plumbing, and the `AlphaTouchInput`
  refactor to cover both ALPHA-register and modal-label cases — left to
  research/planning within the UI-SPEC contract and iOS-gating rule.

## Deferred Ideas

- On-screen ALPHA character grid → v4.2 (input-via-iOS-keyboard chosen instead).
- Annunciator size bump to 13px → only if device legibility testing demands it (non-blocking).
- Bottom-sheet swipe gesture → v4.2 (tap-to-toggle ships in Phase 55).
- Landscape, iPad, Android → v4.2+ (STATE.md Deferred Items).
