---
phase: quick-260603-klp
plan: 01
type: execute
wave: 1
depends_on: []
files_modified:
  - hp41-gui/src/App.css
autonomous: false
requirements: [QUICK-KLP-01]
must_haves:
  truths:
    - "On iPhone, the help-overlay search input is fully below the iOS status bar / Dynamic Island and can be tapped/typed into"
    - "On iPhone, the help-overlay close X button is fully below the status bar and can be tapped"
    - "On desktop window mode, the help-overlay header is visually unchanged (env(safe-area-inset-*) resolves to 0)"
  artifacts:
    - path: "hp41-gui/src/App.css"
      provides: ".help-overlay-header respects env(safe-area-inset-top/left/right)"
      contains: "env(safe-area-inset-top"
  key_links:
    - from: ".help-overlay-header"
      to: "iOS safe-area insets"
      via: "max(base-padding, env(safe-area-inset-top, 0px)) in padding"
      pattern: "env\\(safe-area-inset-top"
---

<objective>
Fix the `?` help overlay ("KEYBOARD SHORTCUTS" / function reference) rendering its search input and close X button hard against the very top edge of the iPhone screen, underneath the iOS status bar / Dynamic Island, making both untappable.

Purpose: The overlay header is currently obscured by the status bar on physical iPhones — the user cannot type into the search field or tap the close X. This is a missing safe-area-inset on the overlay header (gap from Phase 55 touch-ui work, which added `.calculator-safe-area` to the in-flow calculator wrapper but NOT to absolutely-positioned overlays).

Output: A purely CSS-level safe-area fix in `.help-overlay-header` that pushes the header below the notch/Dynamic Island on iOS while leaving desktop window mode visually identical.
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/STATE.md
@./CLAUDE.md

<root_cause>
`.help-overlay` (App.css ~L439) is `position: absolute; top:0; left:0; right:0; bottom:0` and is mounted INSIDE the `.calculator` wrapper (App.tsx L1326, within the `.calculator` div at L1115). The wrapper is `position: relative` (App.css L15) and, on iOS, gains `.calculator-safe-area` padding (App.css L142-147, Phase 55 TOUCH-02).

Critical detail: CSS absolute positioning resolves `top:0` to the **padding box** edge of the positioned ancestor — padding does NOT push absolutely-positioned children inward. So `.calculator-safe-area`'s `padding-top` shifts in-flow flex children down but leaves `.help-overlay` (and its `.help-overlay-header`) pinned to the very top, under the status bar.

Therefore the overlay header must consume the safe-area inset itself. `.help-overlay-header` currently has a flat `padding: 10px 16px` (App.css L457) with no inset awareness.
</root_cause>

<existing_safe_area_pattern>
Phase 55 (TOUCH-02) established the inset method as `env(safe-area-inset-*, 0px)` with `viewport-fit=cover` (already present in hp41-gui/index.html L5). Reuse this — do NOT invent a new utility class. On desktop `env(safe-area-inset-*)` resolves to 0, so `max(base, env(...))` keeps window mode pixel-identical.
</existing_safe_area_pattern>

<interfaces>
Current `.help-overlay-header` rule (App.css ~L453-460):
```css
.help-overlay-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 10px 16px;
  background: var(--panel-header-bg);
  border-bottom: 1px solid var(--panel-border);
}
```
Base padding to preserve: 10px vertical, 16px horizontal.
</interfaces>
</context>

<tasks>

<task type="auto">
  <name>Task 1: Add safe-area insets to the help-overlay header</name>
  <files>hp41-gui/src/App.css</files>
  <action>
Replace the flat `padding: 10px 16px;` in `.help-overlay-header` with safe-area-aware longhand padding so the header clears the iOS status bar / Dynamic Island while staying identical on desktop:

- `padding-top: max(10px, env(safe-area-inset-top, 0px));` — pushes the search input + close X below the notch/Dynamic Island. On desktop the inset is 0, so the effective value stays 10px.
- `padding-left: max(16px, env(safe-area-inset-left, 0px));` — clears the Dynamic Island / rounded corners in landscape.
- `padding-right: max(16px, env(safe-area-inset-right, 0px));` — same for the right inset (and keeps the close X reachable in landscape).
- `padding-bottom: 10px;` — unchanged.

Keep every other declaration in the rule (display:flex, justify-content, align-items, background, border-bottom) exactly as-is. Do NOT add an iOS-only class or any JS gating — `env(safe-area-inset-*, 0px)` is 0 on desktop/window mode, so the `max()` approach leaves desktop rendering byte-for-byte unchanged (CLAUDE.md GUI specifics: window mode must stay unaffected). Add a short comment referencing the Phase 55 TOUCH-02 safe-area pattern and that this covers the absolutely-positioned overlay the wrapper padding cannot reach.

This is a pure GUI frontend CSS change. Do not touch hp41-core, IPC, or any Rust (CLAUDE.md frozen invariants — this is purely layout).
  </action>
  <verify>
    <automated>cd /Users/daniel/GitRepository/hp41-calculator-emulator && grep -A8 '^\.help-overlay-header {' hp41-gui/src/App.css | grep -q 'env(safe-area-inset-top' && grep -A8 '^\.help-overlay-header {' hp41-gui/src/App.css | grep -q 'env(safe-area-inset-left' && grep -A8 '^\.help-overlay-header {' hp41-gui/src/App.css | grep -q 'env(safe-area-inset-right' && echo PASS</automated>
  </verify>
  <done>`.help-overlay-header` uses `max(base, env(safe-area-inset-top/left/right, 0px))` for its top/left/right padding; bottom padding stays 10px; no other declaration changed; no new class or JS gating introduced.</done>
</task>

<task type="auto">
  <name>Task 2: Confirm no regression in GUI build + tests</name>
  <files>hp41-gui/src/App.css</files>
  <action>
Run the GUI frontend test + build gate to confirm the CSS change compiles cleanly and breaks no existing HelpOverlay tests. Use the project task runner per CLAUDE.md (never call cargo/npm directly where a `just` recipe exists). Run `just gui-ci` (or, if that recipe also builds the Rust side and is slow, the frontend-only equivalent the repo defines — check `just --list` for a vitest/lint recipe). The HelpOverlay tests assert DOM structure (header rows, search input, close button) and are CSS-agnostic, so they should pass unchanged; this task is a guard that nothing in the build pipeline rejects the new `max()`/`env()` syntax (lightningcss/Vite must accept it).

Do not modify test files — `env(safe-area-inset-*)` and `max()` are standard CSS that jsdom-based tests do not evaluate, so existing assertions remain valid.
  </action>
  <verify>
    <automated>cd /Users/daniel/GitRepository/hp41-calculator-emulator && just --list 2>/dev/null | grep -qi 'gui' && echo "gui recipes present — run gui test/build gate"</automated>
  </verify>
  <done>GUI frontend test + build gate (`just gui-ci` or the repo's frontend test recipe) passes; no HelpOverlay test regressions; lightningcss/Vite accepts the `max()`/`env()` CSS.</done>
</task>

<task type="checkpoint:human-verify" gate="blocking">
  <what-built>`.help-overlay-header` now consumes `env(safe-area-inset-top/left/right)` via `max()`, pushing the search input and close X below the iOS status bar / Dynamic Island. Desktop window mode is unchanged because the insets resolve to 0.</what-built>
  <how-to-verify>
This is an on-device iOS layout fix — the safe-area inset is non-zero only on real hardware (and approximately in the iOS Simulator), so visual confirmation cannot be automated (CLAUDE.md / Phase 55 D-55.4: authoritative pass is on iPhone).

1. Build + run the GUI on iPhone (or iOS Simulator with a notch/Dynamic Island device, e.g. iPhone 15) via the project's iOS run recipe (`just gui-ios-dev` / the Tauri iOS dev command the repo defines).
2. Open the calculator, press `?` (or the on-screen Help/`?` affordance) to open the help overlay.
3. Confirm the search input field ("Search functions...") is FULLY visible below the status bar / Dynamic Island and can be tapped and typed into.
4. Confirm the close `×` button (top-right) is FULLY visible below the status bar and can be tapped to dismiss the overlay.
5. (Optional, landscape) Rotate to landscape and confirm the close X / search are not clipped by the Dynamic Island on the left/right edges.
6. On macOS desktop window mode, open the same overlay and confirm the header looks visually identical to before (10px/16px padding, no extra top gap).
  </how-to-verify>
  <resume-signal>Type "approved" once the search input and close X are both reachable on iPhone and desktop is unchanged, or describe what is still clipped.</resume-signal>
</task>

</tasks>

<verification>
- `grep` confirms `.help-overlay-header` references `env(safe-area-inset-top`, `-left`, and `-right` via `max()`.
- GUI frontend test/build gate passes with no HelpOverlay regressions.
- On-device: help-overlay search input + close X are below the iOS status bar and tappable; desktop window mode visually unchanged.
</verification>

<success_criteria>
- Help-overlay search input is fully below the iOS status bar / Dynamic Island and accepts taps + text entry on iPhone.
- Help-overlay close X is fully below the status bar and tappable on iPhone.
- Desktop window-mode help-overlay header is pixel-identical to before (insets resolve to 0 via `max()`).
- Change is confined to `hp41-gui/src/App.css`; no Rust / hp41-core / IPC / test files touched; reuses the Phase 55 `env(safe-area-inset-*)` pattern rather than inventing a new utility.
</success_criteria>

<output>
Create `.planning/quick/260603-klp-fix-help-overlay-search-field-rendering-/260603-klp-SUMMARY.md` when done.
</output>
