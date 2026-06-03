---
phase: 55
reviewed: 2026-06-03T00:00:00Z
depth: standard
files_reviewed: 11
files_reviewed_list:
  - hp41-gui/src/App.tsx
  - hp41-gui/src/AlphaTouchInput.tsx
  - hp41-gui/src/Keyboard.tsx
  - hp41-gui/src/BottomSheet.tsx
  - hp41-gui/src/haptics.ts
  - hp41-gui/src/App.css
  - hp41-gui/src-tauri/src/commands.rs
  - hp41-gui/src-tauri/src/lib.rs
  - hp41-core/src/ops/stack_ops.rs
  - hp41-core/src/ops/mod.rs
  - hp41-cli/src/app.rs
findings:
  critical: 0
  warning: 3
  info: 4
  total: 7
status: issues_found
---

# Phase 55: Code Review Report

**Reviewed:** 2026-06-03
**Depth:** standard
**Files Reviewed:** 11
**Status:** issues_found

## Summary

Phase 55 (Touch UI Adaptation) adds iOS touch infrastructure to the GUI and ships
two cross-cutting fidelity fixes. The two highest-risk changes — the keypad-only
name-entry routing in `App.tsx` `handleClick` and the shared `backspace_entry`
core helper — are **correct and well-engineered**.

Verified clean against the project invariants I was asked to check:

- **`backspace_entry` (stack_ops.rs):** infallible, char-safe pops, correct CLX
  end-state, no panics, no `unwrap()`, no `println!`. Lift handling is correct
  (mid-entry lift is already disabled, so the no-op on partial pop is right).
- **CLI↔GUI parity (D-25.6):** both `hp41-cli/src/app.rs:792` and
  `hp41-gui/src-tauri/src/commands.rs:271` call the *same* `backspace_entry`
  helper. No duplication — SC-4 satisfied. The CLI intercept is correctly placed
  *after* the `pending_input.is_some()` return (line 373) and the `alpha_mode`
  return (line 457), so only bare number-entry/idle Backspace changes behavior.
- **`entry_backspace` IPC string** matches between frontend (`App.tsx:179`, `:736`)
  and the Rust prepare-intercept (`commands.rs:271`) — placed before
  `key_map::resolve`, so no missing `Op` variant.
- **Leading-zero `"0."` form** seeded identically in CLI and GUI, with regression
  tests covering `0 . 1` (no `00.1`) and the single-decimal-point invariant.
- **`isTextLabelKind` routing** is correctly scoped — branches (c)/(d) are gated
  on `isTextLabelKind`, so non-text-label modals (fmt, flag, register, single_digit)
  are unaffected. ENTER (`alphaChar='N'`) types 'N'; `←` (`clx_or_a`, no alphaChar)
  correctly falls through to `Backspace`.
- **Invariants:** no `unwrap()`/`println!` added to `hp41-core`; haptics uses the
  bare-string API (`impactFeedback('light')`, no `{ style: ... }`); iOS code gated
  by `#[cfg(mobile)]` / `isIos`; `is_ios` permission + handler + capability wired.

The findings below are quality/robustness issues, not correctness blockers.

## Warnings

### WR-01: `ensureAudioResumed` rejection is unhandled (no internal `.catch()`)

**File:** `hp41-gui/src/haptics.ts:110-119`, call site `hp41-gui/src/App.tsx:1225`
**Issue:** Unlike `triggerHaptic` and `maybeFireErrorHaptic`, which each wrap their
async IPC/await in a silent `.catch()`, `ensureAudioResumed` has **no internal
error guard**. If `await audioCtx.resume()` rejects (iOS can reject `resume()` when
not in a valid gesture context, or when the context is interrupted by an audio
session change), the rejection propagates out of the function. The call site uses
`void ensureAudioResumed(...)` which suppresses the floating-promise lint but does
**not** catch — producing an unhandled promise rejection on iOS. The sibling call
`void triggerHaptic(key, true)` on the next line is safe because `triggerHaptic`
catches internally; the inconsistency makes this easy to miss.
**Fix:** Guard the resume the same way the other helpers guard their awaits:
```ts
export async function ensureAudioResumed(
  audioCtx: AudioContext,
  audioResumedRef: { current: boolean },
): Promise<void> {
  if (audioResumedRef.current) return;
  if (audioCtx.state === 'suspended') {
    await audioCtx.resume().catch(() => {
      // resume can reject outside a valid gesture / on session interruption
    });
  }
  audioResumedRef.current = true;
}
```

### WR-02: `BottomSheet` `emptyText` is dead code — `hasChildren` never false; comment is wrong

**File:** `hp41-gui/src/BottomSheet.tsx:30-58`
**Issue:** The comment claims "React.Children.count handles null/undefined/array
safely" but the code does **not** use `React.Children.count`; it checks
`children !== null && children !== undefined && typeof children !== 'boolean'`.
Both call sites always pass a JSX expression that evaluates to a non-empty **array**
(`{program_steps.map(...)}` for PRGM, and `[printLog.map(...), <div ref=printEndRef/>]`
for PRINT — note the always-present `printEndRef` sentinel `<div>`). An array is
never `null`/`undefined`/`boolean`, so `hasChildren` is **always true**. Result:
the `emptyText` props ("Program memory empty.", "No print output yet.") and the
`.bottom-sheet-empty` branch are unreachable dead code; an empty program/print log
renders a blank scroll area instead of the intended empty-state copy. The PRGM sheet
is also gated `visible={annunciators.prgm}` so it shows whenever PRGM is on, even with
"empty" program memory — the empty-state copy is exactly the case that never fires.
**Fix:** Use the genuine emptiness check the comment promises, e.g.
`const hasChildren = React.Children.count(children) > 0;` (and `import React` or
`import { Children } from 'react'`). For the print sheet, the trailing
`<div ref={printEndRef} />` will still count as a child — either move that sentinel
out of the emptiness check or base the empty-state on `printLog.length === 0`
explicitly at the call site.

### WR-03: print-log auto-scroll silently no-ops on iOS when the bottom sheet is collapsed

**File:** `hp41-gui/src/App.tsx:1070-1073` (effect), `hp41-gui/src/BottomSheet.tsx:50-59`
**Issue:** The auto-scroll effect calls `printEndRef.current?.scrollIntoView(...)`
on every `printLog` change. On iOS the `printEndRef` `<div>` lives inside the
`BottomSheet` `.bottom-sheet-content`, which is `overflow: hidden` with
`max-height: 32px` (peek) while collapsed. New print output therefore does not
scroll into view until the user manually expands the sheet, and `scrollIntoView`
on a node inside an `overflow:hidden` collapsed container can also nudge the
*outer* scroll position unexpectedly. The optional-chaining makes this fail
silently — no error, just missing behavior. Functionally minor (print log is
review-only) but it defeats the "latest line visible" intent on the new primary
platform.
**Fix:** Either auto-expand the print sheet when `printLog` grows (raise
`expanded` via a controlled prop), or skip/guard the `scrollIntoView` when the
sheet is collapsed (`block: 'nearest'` to avoid moving the outer scroll). At
minimum, document that auto-scroll is expansion-gated on iOS.

## Info

### IN-01: `bottomOffset` defaults to `0` and `visualViewport` is unavailable in test/desktop — bar pins to viewport bottom

**File:** `hp41-gui/src/AlphaTouchInput.tsx:69-84`
**Issue:** When `window.visualViewport` is undefined (older WebViews / jsdom),
`updatePosition` never runs and `bottomOffset` stays `0`, so the bar renders at
`bottom: 0px`. That is the correct fallback, but the early `return` inside the
effect means the cleanup also returns `undefined` — fine — yet there is no
fallback to a `window.resize` listener, so a WebView without `visualViewport`
gets no keyboard tracking at all. Acceptable for the iOS-only target (WKWebView
has `visualViewport`), but worth a one-line note since the component is iOS-gated
yet the guard implies broader support.
**Fix (optional):** Add a comment that `visualViewport` is guaranteed on the iOS
WKWebView target, or fall back to a `window.resize` listener for completeness.

### IN-02: `let _ = op_clx(state)` discards an infallible `Result` twice

**File:** `hp41-core/src/ops/stack_ops.rs:56, 62`
**Issue:** `op_clx` is infallible (always returns `Ok(())`) but typed
`Result<(), HpError>`. `backspace_entry` discards it with `let _ = op_clx(state)`.
This is correct and idiomatic given the signature, but `let _ =` silently swallows
a future error if `op_clx` ever becomes fallible. Low risk given the frozen CLX
semantics.
**Fix (optional):** Either keep `backspace_entry` returning `Result<(), HpError>`
and propagate with `op_clx(state)?`, or add a `debug_assert!(result.is_ok())`.
Current form is acceptable.

### IN-03: ALPHA-mode `handleChange` diff logic relies on `inputValue` always being `''`

**File:** `hp41-gui/src/AlphaTouchInput.tsx:99-116`
**Issue:** In ALPHA mode the handler computes `diff = newValue.slice(inputValue.length)`
then immediately `setInputValue('')`. This works only because `inputValue` is
invariably `''` in ALPHA mode (cleared after every change). The `slice(inputValue.length)`
is therefore always `slice(0)` — the diff machinery is redundant in this mode and
could confuse a future maintainer into thinking accumulation occurs. iOS
autocorrect/predictive insertion that replaces the whole field would still be
handled char-by-char, which is the desired outcome, so no bug today.
**Fix (optional):** Simplify the ALPHA branch to iterate `newValue` directly, or
add a comment that `inputValue` is guaranteed empty here.

### IN-04: `printEndRef` / `activeStepRef` attached in two mutually-exclusive subtrees

**File:** `hp41-gui/src/App.tsx:1254-1269` & `:1294-1320` (and the prgm pair)
**Issue:** `printEndRef` is referenced by both the iOS `BottomSheet` print children
and the desktop `.print-panel`; `activeStepRef` likewise in both PRGM renders. They
are mutually exclusive via the `isIos` ternary, so only one ever mounts and React
assigns the ref to the live node — no double-attach bug. Noted only because a future
refactor that removes the `isIos` exclusivity (e.g. rendering both for a tablet
split view) would create a ref collision where the last-rendered node wins.
**Fix (optional):** None required now; keep the `isIos` ternary exclusivity if these
refs stay shared.

---

_Reviewed: 2026-06-03_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_

## Findings Summary

| Severity | Count |
|----------|-------|
| Critical | 0 |
| Warning  | 3 |
| Info     | 4 |
| **Total**| **7** |
