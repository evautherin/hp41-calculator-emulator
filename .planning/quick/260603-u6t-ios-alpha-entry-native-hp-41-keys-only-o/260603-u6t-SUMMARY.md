---
phase: quick-260603-u6t
plan: inline
subsystem: gui-frontend
tags: [ios, alpha-entry, keys-only, touch-target, alpha-backspace, hp41-native]
key_files:
  modified:
    - hp41-gui/src/App.tsx
    - hp41-gui/src/App.test.tsx
    - hp41-gui/src/Keyboard.tsx
metrics:
  completed: "2026-06-03"
verification:
  on_device: "PASS (iPhone 15 Pro) — ALPHA entry via on-screen blue keys (no iOS keyboard); ← deletes last char; ENTER's 'N' tappable; FUNCTION NAME? keys-only. Desktop Backspace deletes alpha char."
---

# Quick Task 260603-u6t: Native keys-only ALPHA entry on iOS

Owner decision: ALPHA-register (and function-name) entry on iOS uses ONLY the on-screen
HP-41 blue-letter keys — the iOS software keyboard is removed entirely. Plus the matching
backspace fixes the GUI lacked vs the CLI.

## Changes (all hp41-gui frontend)
1. **On-screen ← in ALPHA mode** → `alpha_backspace` (delete last char; HP-41 ← key) instead
   of `alpha_clear` (full wipe). `handleClick` clx_or_a branch. (bf45e05)
2. **Physical Backspace in ALPHA mode** → `alpha_backspace` (was `entry_backspace`, which
   clears the number-entry buffer). `resolveKeyId`, before the MAP. Matches CLI D-13. Fixes
   the desktop deletion the owner reported. (bf45e05)
3. **No iOS keyboard for ALPHA**: removed `annunciators.alpha` from the AlphaTouchInput
   gating (bf45e05), then **removed the bar entirely** including the `modal_requires_alpha_label`
   FUNCTION NAME? case (32121ca) — the collect-for-modal pendingInput already routes blue-key
   letters into the name accumulator and ALPHA submits via the SAME `submit_modal_with_label`
   IPC (`__submit_modal_with_label__<acc>` magic prefix, Phase-31 design). `AlphaTouchInput.tsx`
   is now unused (dead-code cleanup later). ALPHA text shows on the main 14-seg display.
4. **iOS touch targets sized to the actual key** (65e7a3a): `.key-touch-target` was a fixed
   44px centered on each key's midpoint; the wide ENTER key (colSpan 2) thus had only a 44px
   centre target, making its ALPHA letter **'N' nearly un-tappable** once keys-only. Now
   `width`/`height` come from `keyPosition()`; CSS `min-width/height:44px` keeps small keys
   HIG-compliant. iOS-only (desktop uses the full SVG click target).

## Tests
295 vitest pass. Updated L3 (← → alpha_backspace), M3b (plain ALPHA → bar absent),
M3c (modal-label → bar absent); AlphaTouchInput backspace test → alpha_backspace.

## Follow-ups (noted, not done)
- `AlphaTouchInput.tsx` + `.alpha-touch-input-bar` CSS are now dead code — remove in a cleanup.
- See [[reference_ios_gui_layout_gotchas]].
