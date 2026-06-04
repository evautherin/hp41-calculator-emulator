# Phase 55: Touch UI Adaptation — Pattern Map

**Mapped:** 2026-06-03
**Files analyzed:** 14 new/modified files
**Analogs found:** 13 / 14

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|---|---|---|---|---|
| `hp41-gui/src-tauri/src/commands.rs` | backend command | request-response | `commands.rs:544` `is_macos()` | exact |
| `hp41-gui/src-tauri/permissions/is-ios.toml` | config | — | `permissions/is-macos.toml` | exact |
| `hp41-gui/src-tauri/capabilities/default.json` | config | — | `capabilities/default.json` (current) | exact |
| `hp41-gui/src-tauri/src/lib.rs` | config / plugin registry | — | `lib.rs` `#[cfg(desktop)]` autostart block | exact |
| `hp41-gui/src-tauri/Cargo.toml` | config | — | existing `[dependencies]` block; autostart desktop-gate pattern | role-match |
| `hp41-gui/package.json` | config | — | existing `package.json` `dependencies` block | exact |
| `hp41-gui/src/App.tsx` | component (root) | request-response | `App.tsx:494-495` `isMacos` pattern | exact |
| `hp41-gui/src/AlphaTouchInput.tsx` | component | request-response | `SettingsPanel.tsx` (prop-driven panel) + `App.tsx` modal-label dispatch path | role-match |
| `hp41-gui/src/AlphaTouchInput.test.tsx` | test | — | `Keyboard.test.tsx` (render harness) + `App.test.tsx` (invoke mock + `makeEmptyView`) | role-match |
| `hp41-gui/src/BottomSheet.tsx` | component | event-driven | `SettingsPanel.tsx` (conditional render + prop-driven) | role-match |
| `hp41-gui/src/BottomSheet.test.tsx` | test | — | `SettingsPanel.test.tsx` (open/closed conditional render tests) | role-match |
| `hp41-gui/src/Keyboard.tsx` | component | request-response | `Keyboard.tsx` `handleKeyClick` + `.key-pressed` | exact |
| `hp41-gui/src/App.css` | style | — | `App.css` `.key`, `.key-pressed`, `.print-panel` blocks | exact |
| `hp41-gui/index.html` | config | — | `index.html` viewport meta line 5 | exact |
| `hp41-gui/src-tauri/gen/apple/project.yml` | config (iOS) | — | `project.yml` `UISupportedInterfaceOrientations` lines 50-58 | exact |
| `hp41-gui/src-tauri/gen/apple/hp41-gui_iOS/Info.plist` | config (iOS) | — | `Info.plist` `UISupportedInterfaceOrientations` lines 30-35 | exact |

---

## Pattern Assignments

### `hp41-gui/src-tauri/src/commands.rs` — ADD `is_ios()` command

**Analog:** `commands.rs` lines 538–546 (`is_macos`)

**Exact clone pattern** (lines 538–546):
```rust
/// Tauri command: report whether the backend was compiled for macOS.
///
/// The frontend uses this to render the macOS-only launch-mode control. The launch-mode
/// preference has no effect on Windows/Linux (the `setup()` branch is `cfg(macos)`), so
/// showing the control there would mislead the user.
#[tauri::command]
pub fn is_macos() -> bool {
    cfg!(target_os = "macos")
}
```

**New command to add** (after line 546, before `save_state`):
```rust
/// Tauri command: report whether the backend was compiled for iOS.
///
/// The frontend uses this to gate touch-specific behaviors (bottom sheets,
/// collapsible stack panel, AlphaTouchInput bar, .key-touch-target overlays,
/// haptic calls). Authoritative via compile-time `cfg(target_os = "ios")` —
/// consistent with the existing `is_macos()` precedent (D-55.1).
#[tauri::command]
pub fn is_ios() -> bool {
    cfg!(target_os = "ios")
}
```

**invoke_handler registration** (analog: line 181 `commands::is_macos`):
```rust
commands::is_ios,   // D-55.1 — iOS platform detection (mirrors is_macos)
```

---

### `hp41-gui/src-tauri/permissions/is-ios.toml` — NEW file

**Analog:** `permissions/is-macos.toml` (entire file, 6 lines)

**Entire file content** (exact clone with `ios` substituted for `macos`):
```toml
"$schema" = "../gen/schemas/desktop-schema.json"

[[permission]]
identifier = "allow-is-ios"
description = "Allows the is_ios command."
commands.allow = ["is_ios"]
```

**Ordering discipline (CLAUDE.md Tauri v2.11 rule):** Run `cargo check` from `hp41-gui/src-tauri/` BEFORE editing `default.json`. The check regenerates `gen/schemas/desktop-schema.json` which the TOML `$schema` points to, and registers the `allow-is-ios` identifier in the permission registry.

---

### `hp41-gui/src-tauri/capabilities/default.json` — ADD entries

**Analog:** `capabilities/default.json` lines 1–31 (current file)

**Current file** (lines 1–31):
```json
{
  "identifier": "default",
  "description": "Default capability for hp41-gui — core + Phase 14 IPC commands",
  "windows": ["main"],
  "permissions": [
    "core:default",
    ...
    "allow-is-macos",
    ...
    "autostart:default"
  ]
}
```

**Entries to add** (alongside `"allow-is-macos"` line 20):
```json
"allow-is-ios",
"haptics:allow-impact-feedback",
"haptics:allow-notification-feedback",
"haptics:allow-selection-feedback",
"haptics:allow-vibrate"
```

The four `haptics:*` identifiers are published by `tauri-plugin-haptics` when registered. They only have effect on mobile builds; on desktop they are present but inactive.

---

### `hp41-gui/src-tauri/src/lib.rs` — ADD haptics plugin registration

**Analog:** `lib.rs` lines 46–50 (`#[cfg(desktop)]` autostart block)

**Existing desktop-gate pattern** (lines 46–50):
```rust
// tauri-plugin-autostart is desktop-only: its `init`/`MacosLauncher` symbols do
// not exist on the iOS/Android mobile targets, so registering it unconditionally
// breaks the `aarch64-apple-ios` cross-compile (v4.1 Phase 53 scaffold spike,
// P-iOS-08 smoke). Gate it to desktop — desktop behavior is unchanged.
#[cfg(desktop)]
let builder = builder.plugin(tauri_plugin_autostart::init(
    tauri_plugin_autostart::MacosLauncher::LaunchAgent,
    None,
));
```

**New mobile-gate block** (add after the `#[cfg(desktop)]` block, before `.setup(|app|`):
```rust
// tauri-plugin-haptics is mobile-only: iOS UIImpactFeedbackGenerator symbols
// do not exist on desktop targets. Gating with #[cfg(mobile)] mirrors the
// #[cfg(desktop)] pattern for autostart above. The JS haptic calls are
// iOS-gated via isIos, so desktop code never reaches the IPC endpoint.
#[cfg(mobile)]
let builder = builder.plugin(tauri_plugin_haptics::init());
```

**Critical:** `#[cfg(mobile)]` — NOT `#[cfg(target_os = "ios")]`. The `mobile` cfg alias covers both iOS and Android. Unconditional registration breaks desktop compile (RESEARCH Pitfall 2, cfg(mobile) blind spot per MEMORY.md).

---

### `hp41-gui/src-tauri/Cargo.toml` — ADD haptics dependency

**Analog:** `Cargo.toml` lines 19–26 (`[dependencies]` block) + the existing conditional dependency structure

**Current `[dependencies]` block** (lines 19–26):
```toml
[dependencies]
tauri = { version = "2.11", features = ["macos-private-api", "tray-icon", "image-png"] }
tauri-plugin-dialog = "2"
tauri-plugin-autostart = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
hp41-core = { path = "../../hp41-core" }
dirs = "6"
```

**New section to add** (after `[dependencies]`, before `[dev-dependencies]`):
```toml
[target.'cfg(any(target_os = "android", target_os = "ios"))'.dependencies]
tauri-plugin-haptics = "2.3.2"
```

Do NOT add `tauri-plugin-haptics` to `[dependencies]` unconditionally — that includes dead code on desktop and may cause linker errors.

---

### `hp41-gui/package.json` — ADD haptics npm package

**Analog:** `package.json` lines 13–17 (`dependencies` block)

**Current dependencies block** (lines 13–17):
```json
"dependencies": {
  "@tauri-apps/api": "^2.11",
  "@tauri-apps/plugin-dialog": "^2.7",
  "@tauri-apps/plugin-haptics": "^2.3.2",
  "react": "^19.2",
  "react-dom": "^19.2"
},
```

The `@tauri-apps/plugin-haptics` entry is already present in `package.json` (installed during research phase). Verify it is present; no action needed if already there.

---

### `hp41-gui/src/App.tsx` — ADD `isIos` state, haptic/audio plumbing, error-haptic guard

**Analogs:**
- `App.tsx:277` — `isMacos` state declaration
- `App.tsx:494-496` — `is_macos` useEffect consumer
- `App.tsx:699-708` — post-IPC `setCalcState(view)` pattern (error-haptic hook point)
- `App.tsx:1131-1138` — `<Keyboard>` render (overlay prop injection point)

**isMacos state declaration pattern** (line 277):
```typescript
const [isMacos, setIsMacos] = useState(false);
```

**New isIos state** (add alongside line 277):
```typescript
const [isIos, setIsIos] = useState(false);
```

**isMacos useEffect pattern** (lines 494–496):
```typescript
useEffect(() => {
  invoke<boolean>('is_macos').then(setIsMacos).catch(() => setIsMacos(false));
}, []);
```

**New isIos useEffect** (add after lines 494–496):
```typescript
useEffect(() => {
  invoke<boolean>('is_ios').then(setIsIos).catch(() => setIsIos(false));
}, []);
```

**Post-IPC setCalcState pattern** (lines 699–708 in `handleClick`):
```typescript
try {
  let view: CalcStateView;
  // ...
  view = await invokeForKey(effectiveId, calcState);
  setCalcState(view);
  setErrorMessage(null);
} catch (err) {
  showToast(extractErrMessage(err));
} finally {
  // ...
}
```

**Error-haptic guard** (add `useRef` near line 293 with other refs, and hook after `setCalcState(view)`):
```typescript
// Error haptic guard — prevents re-firing on every render while error is displayed.
const errorHapticFiredRef = useRef(false);

// In each post-IPC .then() or try block, after setCalcState(view):
const isError = view.display_str.includes('DATA ERROR') || view.display_str.includes('NO ROOM');
if (isIos && isError && !errorHapticFiredRef.current) {
  errorHapticFiredRef.current = true;
  notificationFeedback('error').catch(() => {/* silent on desktop */});
} else if (!isError) {
  errorHapticFiredRef.current = false;
}
```

**AlphaTouchInput render point** (after `<Keyboard>` at line 1131):
```tsx
{isIos && (calcState.annunciators.alpha || calcState.modal_requires_alpha_label) && (
  <AlphaTouchInput
    isAlphaMode={calcState.annunciators.alpha}
    isModalLabelMode={calcState.modal_requires_alpha_label}
    modalPrompt={calcState.modal_prompt}
    onDispatch={dispatchKeyId}
    onSubmitLabel={(label) => invoke('submit_modal_with_label', { label })}
  />
)}
```

**BottomSheet render points** (add after `<Keyboard>`, iOS-gated):
```tsx
{isIos && (
  <BottomSheet
    id="print-sheet"
    title="PRINT LOG"
    visible={printLog.length > 0}
  >
    {printLog.map((line, i) => <div key={i} className="print-line">{line}</div>)}
  </BottomSheet>
)}
{isIos && calcState.annunciators.prgm && (
  <BottomSheet
    id="prgm-sheet"
    title="PROGRAM"
    visible={true}
  >
    {calcState.program_steps.map((step, i) => (
      <div key={i} ref={calcState.pc === i ? activeStepRef : null}
        className={`step-row${calcState.pc === i ? ' step-active' : ''}`}>{step}</div>
    ))}
  </BottomSheet>
)}
```

---

### `hp41-gui/src/AlphaTouchInput.tsx` — NEW component

**Analogs:**
- `SettingsPanel.tsx` lines 1–64 — props interface + conditional return null + useEffect + `useRef` pattern
- `App.tsx:56-66` — `modal_requires_alpha_label` + `modal_prompt` CalcStateView fields (the two trigger conditions)
- `App.tsx:101-103` — `SUBMIT_MODAL_WITH_LABEL_PREFIX` routing (the modal-label submit path to reuse)
- `App.tsx:153-158` — `alpha_<X>` routing (the ALPHA-register dispatch path to reuse)
- RESEARCH.md Pattern 5 — `visualViewport` keyboard-tracking pattern

**Props interface** (from RESEARCH.md §D-55.2 Modal-Label Touch Entry):
```typescript
interface AlphaTouchInputProps {
  isAlphaMode: boolean;           // calcState.annunciators.alpha
  isModalLabelMode: boolean;      // calcState.modal_requires_alpha_label
  modalPrompt: string | null;     // calcState.modal_prompt
  onDispatch: (keyId: string) => void;    // App.tsx dispatchKeyId
  onSubmitLabel: (label: string) => void; // invoke('submit_modal_with_label')
}
```

**SettingsPanel early-return pattern** (SettingsPanel.tsx line 64):
```typescript
if (!open) return null;
```

**visualViewport keyboard-tracking pattern** (RESEARCH.md Pattern 5):
```typescript
useEffect(() => {
  const vv = window.visualViewport;
  if (!vv) return;
  const updatePosition = () => {
    const keyboardHeight = window.innerHeight - vv.height - vv.offsetTop;
    setBottomOffset(Math.max(keyboardHeight, 0));
  };
  vv.addEventListener('resize', updatePosition);
  vv.addEventListener('scroll', updatePosition);
  return () => {
    vv.removeEventListener('resize', updatePosition);
    vv.removeEventListener('scroll', updatePosition);
  };
}, []);
```

**alpha_<X> dispatch path** (App.tsx lines 153–158):
```typescript
if (state?.annunciators?.alpha && e.key.length === 1) {
  const ch = e.key.toUpperCase();
  if (/^[A-Z0-9 ]$/.test(ch)) {
    return `alpha_${ch}`;
  }
}
```

**submit_modal_with_label dispatch path** (App.tsx lines 101–103):
```typescript
if (effectiveId.startsWith(SUBMIT_MODAL_WITH_LABEL_PREFIX)) {
  const label = effectiveId.slice(SUBMIT_MODAL_WITH_LABEL_PREFIX.length);
  return invoke<CalcStateView>('submit_modal_with_label', { label });
}
```

**16px font-size constraint** (non-negotiable — RESEARCH Pitfall 5):
```css
.alpha-touch-input-bar input {
  font-size: 16px; /* MUST NOT be below 16px — iOS auto-zoom prevention */
}
```

---

### `hp41-gui/src/AlphaTouchInput.test.tsx` — NEW test file

**Analog:** `Keyboard.test.tsx` lines 1–38 (render harness pattern) + `App.test.tsx` lines 30–107 (invoke mock + `makeEmptyView`)

**vi.mock pattern** (App.test.tsx lines 30–34):
```typescript
const mockInvoke = vi.fn();
vi.mock('@tauri-apps/api/core', () => ({
  invoke: (cmd: string, args?: unknown) => mockInvoke(cmd, args),
}));
```

**Also mock haptics plugin** (pattern for new plugin):
```typescript
vi.mock('@tauri-apps/plugin-haptics', () => ({
  impactFeedback: vi.fn().mockResolvedValue(undefined),
  notificationFeedback: vi.fn().mockResolvedValue(undefined),
}));
```

**TestHarness pattern** (Keyboard.test.tsx lines 20–38):
```typescript
function TestHarness({ isAlphaMode, isModalLabelMode, ... }) {
  return (
    <AlphaTouchInput
      isAlphaMode={isAlphaMode}
      isModalLabelMode={isModalLabelMode}
      modalPrompt={null}
      onDispatch={vi.fn()}
      onSubmitLabel={vi.fn()}
    />
  );
}
```

**Key test coverage** (per RESEARCH.md Validation Architecture):
- Renders when `isAlphaMode=true`
- Renders when `isModalLabelMode=true`
- Does NOT render when both false
- Dispatches `alpha_A` when `isAlphaMode=true` and input changes to "A"
- Calls `onSubmitLabel` on Done when `isModalLabelMode=true`
- Dispatches `clx` on Backspace when `isAlphaMode=true`

---

### `hp41-gui/src/BottomSheet.tsx` — NEW component

**Analog:** `SettingsPanel.tsx` lines 42–145 — prop-driven panel with conditional render

**SettingsPanel props + conditional render pattern** (SettingsPanel.tsx lines 42–64):
```typescript
export function SettingsPanel({
  open, onClose, currentTheme, onThemeChange, ...
}: SettingsPanelProps) {
  const panelRef = useRef<HTMLDivElement>(null);
  // ...
  if (!open) return null;

  return (
    <div ref={panelRef} className="settings-panel" role="dialog" aria-label="Settings">
      ...
    </div>
  );
}
```

**BottomSheet props shape** (iOS-gated pull-up panel):
```typescript
interface BottomSheetProps {
  id: string;             // "print-sheet" | "prgm-sheet"
  title: string;          // "PRINT LOG" | "PROGRAM"
  visible: boolean;       // controls whether sheet appears
  children: React.ReactNode;
}
```

**CSS class toggle pattern** (state-driven className):
```typescript
const [expanded, setExpanded] = useState(false);
// ...
<div className={`bottom-sheet${expanded ? ' expanded' : ''}`}>
```

**Existing print-panel-header pattern** (App.css lines 137–148, App.tsx lines 1161–1163) — reuse for sheet header structure:
```tsx
<div className="bottom-sheet-header">
  <span className="bottom-sheet-title">{title}</span>
  <button className="bottom-sheet-handle" onClick={() => setExpanded(e => !e)} aria-label="Toggle panel" />
</div>
```

---

### `hp41-gui/src/BottomSheet.test.tsx` — NEW test file

**Analog:** `SettingsPanel.test.tsx` lines 39–54 (open/closed conditional render tests)

**Conditional render test pattern** (SettingsPanel.test.tsx lines 39–53):
```typescript
it('renders null when open=false', () => {
  const { container } = render(<SettingsPanel open={false} ... />);
  expect(container.firstChild).toBeNull();
});

it('renders panel when open=true', () => {
  const { container } = render(<SettingsPanel open={true} ... />);
  expect(container.firstChild).not.toBeNull();
});
```

**afterEach cleanup pattern** (SettingsPanel.test.tsx lines 31–33):
```typescript
import { describe, it, expect, vi, afterEach } from 'vitest';
afterEach(cleanup);
```

**Key test coverage** (per RESEARCH.md Validation Architecture):
- `visible=false` → renders null (iOS desktop regression guard)
- `visible=true, isIos=true` → renders sheet
- `visible=true, isIos=false` → does not render (the `isIos` gate is in App.tsx, not BottomSheet, so BottomSheet itself always renders when mounted — but App.tsx must gate mounting)
- Expand/collapse: clicking drag handle toggles `expanded` class

---

### `hp41-gui/src/Keyboard.tsx` — ADD `.key-touch-target` overlays + `onPointerDown`

**Analog:** `Keyboard.tsx` lines 323–329 (`handleKeyClick` + `pressedKey` state)

**Existing `handleKeyClick` pattern** (lines 323–329):
```typescript
const handleKeyClick = (key: KeyDef) => {
  if (!key.id) return;
  if (busyRef.current) return;
  setPressedKey(key.id);
  setTimeout(() => setPressedKey(prev => (prev === key.id ? null : prev)), 150);
  onKey(key);
};
```

**Existing onClick on SVG `<g>` elements** (lines 398–409):
```tsx
<g
  key={key.id || `key-${key.row}-${key.col}`}
  className={`key${pressedKey === key.id ? ' key-pressed' : ''}`}
  onClick={() => handleKeyClick(key)}
>
```

**New prop to add** (alongside existing props at line 280):
```typescript
isIos?: boolean;     // gates .key-touch-target overlay rendering
onPointerDown?: (key: KeyDef) => void;  // haptic + audio trigger
```

**keyPosition() function** (lines 242–268) — already returns `{ x, y, w, h }` in SVG design units. For percentage-based overlay positioning:
```typescript
// cx_pct = (x + w/2) / KEYBOARD_W * 100
// cy_pct = (y + h/2) / KEYBOARD_H * 100
```

**Touch overlay element** (add as sibling of `<svg>`, inside a `position: relative` wrapper):
```tsx
{isIos && key.id && (
  <div
    className="key-touch-target"
    style={{
      left: `${(pos.x + pos.w / 2) / KEYBOARD_W * 100}%`,
      top:  `${(pos.y + pos.h / 2) / KEYBOARD_H * 100}%`,
    }}
    onPointerDown={() => onPointerDown?.(key)}
    onClick={() => handleKeyClick(key)}
    aria-label={key.label}
  />
)}
```

The SVG's existing `onClick` on `<g>` must be disabled (or `busyRef` deduplicated) on iOS to prevent double-fire. Cleanest: pass `isIos` and skip the SVG `onClick` when `isIos && key.id` (overlay handles dispatch).

---

### `hp41-gui/src/App.css` — ADD touch CSS classes

**Analog:** `App.css` lines 102–123 (`.key`, `.key-pressed`, `.key:hover` block) + lines 125–158 (`.print-panel` block)

**Existing key animation block** (lines 102–123):
```css
.key {
  cursor: pointer;
  pointer-events: all;
  transform-box: fill-box;
  transform-origin: center;
  transition: transform 80ms ease-out;
}
.key-pressed {
  transform: scale(0.92);
}
.key:hover:not(.key-pressed) {
  opacity: 0.85;
}
```

**New classes to add** (per UI-SPEC "New CSS Classes Summary"):
```css
/* ── Phase 55: Touch Adaptation Layer ──────────────────────────────────── */

/* Hit-target overlay — 44pt minimum, transparent, no tap flash */
.key-touch-target {
  position: absolute;
  min-width: 44px;
  min-height: 44px;
  touch-action: manipulation;
  -webkit-tap-highlight-color: transparent;
  cursor: pointer;
  transform: translate(-50%, -50%);
  /* top and left set per-key via inline style as % of SVG container */
  background: transparent;
  border: none;
}

/* Safe-area padding applied to the calculator wrapper on iOS */
.calculator-safe-area {
  padding-top: env(safe-area-inset-top, 0px);
  padding-bottom: env(safe-area-inset-bottom, 0px);
  padding-left: env(safe-area-inset-left, 0px);
  padding-right: env(safe-area-inset-right, 0px);
}

/* ALPHA touch input bar — fixed above iOS keyboard */
.alpha-touch-input-bar {
  position: fixed;
  left: 0;
  right: 0;
  z-index: 80; /* above help/wizard/settings (60-70) */
  background: var(--panel-bg);
  border-top: 1px solid var(--panel-border);
  padding: 12px 16px;
  display: flex;
  align-items: center;
  gap: 8px;
}

.alpha-touch-input-bar input {
  font-size: 16px; /* MUST NOT go below 16px — iOS auto-zoom prevention */
  flex: 1;
  background: var(--display-bg);
  color: var(--display-text);
  border: 1px solid var(--panel-border);
  padding: 4px 12px;
  font-family: 'Courier New', Courier, monospace;
}

.alpha-touch-input-label {
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.1em;
  text-transform: uppercase;
  color: var(--text-muted);
}

/* Bottom sheet — pull-up panel for print and PRGM on iOS */
.bottom-sheet {
  position: fixed;
  bottom: 0;
  left: 0;
  right: 0;
  max-height: 32px; /* peek height (collapsed) */
  overflow: hidden;
  background: var(--panel-bg);
  border-top: 1px solid var(--panel-border);
  border-radius: 8px 8px 0 0;
  z-index: 50;
  transition: max-height 300ms ease;
}

.bottom-sheet.expanded {
  max-height: 40vh;
}

.bottom-sheet-header {
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 16px;
  cursor: pointer;
  -webkit-tap-highlight-color: transparent;
}

.bottom-sheet-title {
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.1em;
  text-transform: uppercase;
  color: var(--text-secondary);
}

.bottom-sheet-handle {
  width: 32px;
  height: 48px; /* 2xl — 48pt tap zone per UI-SPEC */
  display: flex;
  align-items: center;
  justify-content: center;
  background: none;
  border: none;
  color: var(--text-muted);
  cursor: pointer;
  touch-action: manipulation;
  -webkit-tap-highlight-color: transparent;
}

.bottom-sheet-content {
  overflow-y: auto;
  overscroll-behavior-y: contain;
  padding: 0 16px 16px;
  max-height: calc(40vh - 32px);
  font-family: 'Courier New', Courier, monospace;
  font-size: 13px;
}

/* Collapsible stack panel (iOS only) */
.stack-panel-collapsible {
  overflow: hidden;
  transition: max-height 200ms ease;
  max-height: 200px;
}

.stack-panel-collapsible.collapsed {
  max-height: 0;
}

.stack-panel-toggle {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 4px 10px;
  background: var(--panel-bg);
  cursor: pointer;
  touch-action: manipulation;
  -webkit-tap-highlight-color: transparent;
  border-bottom: 1px solid var(--display-border);
  font-size: 11px;
  color: var(--text-muted);
}
```

**P51 invariant preserved:** `.key` and `.key-pressed` (transform-box: fill-box) must NEVER be moved to themes.css. All new classes above are in App.css, not themes.css.

---

### `hp41-gui/index.html` — ADD `viewport-fit=cover`

**Analog:** `index.html` line 5 (current viewport meta)

**Current line 5**:
```html
<meta name="viewport" content="width=device-width, initial-scale=1.0" />
```

**Replacement**:
```html
<meta name="viewport" content="width=device-width, initial-scale=1.0, viewport-fit=cover" />
```

`viewport-fit=cover` is required for `env(safe-area-inset-*)` CSS to be populated by WKWebView. Without it the safe-area values are always 0.

---

### `hp41-gui/src-tauri/gen/apple/project.yml` — RESTRICT to portrait-only

**Analog:** `project.yml` lines 50–58 (`UISupportedInterfaceOrientations` block)

**Current block** (lines 50–58):
```yaml
UISupportedInterfaceOrientations:
  - UIInterfaceOrientationPortrait
  - UIInterfaceOrientationLandscapeLeft
  - UIInterfaceOrientationLandscapeRight
UISupportedInterfaceOrientations~ipad:
  - UIInterfaceOrientationPortrait
  - UIInterfaceOrientationPortraitUpsideDown
  - UIInterfaceOrientationLandscapeLeft
  - UIInterfaceOrientationLandscapeRight
```

**Replacement** (portrait lock per TOUCH-02 / UI-SPEC):
```yaml
UISupportedInterfaceOrientations:
  - UIInterfaceOrientationPortrait
UISupportedInterfaceOrientations~ipad:
  - UIInterfaceOrientationPortrait
  - UIInterfaceOrientationPortraitUpsideDown
  - UIInterfaceOrientationLandscapeLeft
  - UIInterfaceOrientationLandscapeRight
```

Leave the `~ipad` key unchanged (iPad landscape deferred to v4.2 per UI-SPEC).

**CRITICAL — RESEARCH Pitfall 6:** After editing `project.yml`, ALSO edit `Info.plist` directly (see below), because `Info.plist` is a generated artifact that may not be regenerated without a full `tauri ios init` run.

---

### `hp41-gui/src-tauri/gen/apple/hp41-gui_iOS/Info.plist` — RESTRICT to portrait-only

**Analog:** `Info.plist` lines 30–35 (`UISupportedInterfaceOrientations` array)

**Current block** (lines 30–35):
```xml
<key>UISupportedInterfaceOrientations</key>
<array>
  <string>UIInterfaceOrientationPortrait</string>
  <string>UIInterfaceOrientationLandscapeLeft</string>
  <string>UIInterfaceOrientationLandscapeRight</string>
</array>
```

**Replacement**:
```xml
<key>UISupportedInterfaceOrientations</key>
<array>
  <string>UIInterfaceOrientationPortrait</string>
</array>
```

Leave `UISupportedInterfaceOrientations~ipad` array (lines 36–43) unchanged.

---

## Shared Patterns

### iOS Platform Gate (`isIos`)
**Source:** `App.tsx:277` (`isMacos` pattern), `App.tsx:494-496` (`is_macos` useEffect)
**Apply to:** All iOS-specific JSX rendering in `App.tsx`, `Keyboard.tsx`, `AlphaTouchInput.tsx`, `BottomSheet.tsx`

```typescript
// State declaration (alongside isMacos):
const [isIos, setIsIos] = useState(false);

// One-shot useEffect consumer (alongside is_macos block):
useEffect(() => {
  invoke<boolean>('is_ios').then(setIsIos).catch(() => setIsIos(false));
}, []);

// Usage gate pattern:
{isIos && <IOSOnlyComponent ... />}
if (isIos) { /* touch-specific code */ }
```

### Tauri v2.11 Command Registration
**Source:** `lib.rs:166-188` (invoke_handler), `commands.rs:538-546` (is_macos body)
**Apply to:** `is_ios` command
**Rule:** New command → TOML in `permissions/` → `default.json` entry → `cargo check` → `invoke_handler![]` entry.

### Haptic API (verified correct form)
**Source:** RESEARCH.md Pattern 3 (FIXES UI-SPEC ERROR)
**Apply to:** All haptic call sites in `App.tsx`, `Keyboard.tsx`

```typescript
import { impactFeedback, notificationFeedback } from '@tauri-apps/plugin-haptics';

// CORRECT lowercase string literals:
await impactFeedback('light');   // NOT { style: 'Light' }
await impactFeedback('medium');
await impactFeedback('heavy');
await notificationFeedback('error'); // NOT { type: 'Error' }

// All haptic calls must be iOS-gated + silently catch desktop:
if (isIos) {
  await impactFeedback(tier).catch(() => {});
}
```

### Mobile-Only Plugin Registration
**Source:** `lib.rs:46-50` (`#[cfg(desktop)]` autostart analog)
**Apply to:** `tauri_plugin_haptics::init()` in `lib.rs`

```rust
// Pattern: cfg-gate mobile-only plugins to avoid desktop link failure
#[cfg(mobile)]
let builder = builder.plugin(tauri_plugin_haptics::init());
```

### cfg(mobile) Blind-Spot Guard
**Source:** MEMORY.md `project_cfg_mobile_gate_blindspot.md`, RESEARCH.md Pitfall 8
**Apply to:** Any code inside `#[cfg(mobile)]` blocks after writing
**Verification command:**
```bash
cargo check --target aarch64-apple-ios
# run from hp41-gui/src-tauri/
```

### Vitest Mock Pattern for Plugin Modules
**Source:** `App.test.tsx:30-34` (invoke mock)
**Apply to:** `AlphaTouchInput.test.tsx`, `BottomSheet.test.tsx`

```typescript
// Mock Tauri core invoke:
const mockInvoke = vi.fn();
vi.mock('@tauri-apps/api/core', () => ({
  invoke: (cmd: string, args?: unknown) => mockInvoke(cmd, args),
}));

// Mock haptics plugin (new — required for any test that imports AlphaTouchInput or App):
vi.mock('@tauri-apps/plugin-haptics', () => ({
  impactFeedback: vi.fn().mockResolvedValue(undefined),
  notificationFeedback: vi.fn().mockResolvedValue(undefined),
}));
```

### touch-action: manipulation (tap-delay elimination)
**Source:** RESEARCH.md Pattern 7 (tap-delay), UI-SPEC §Tap Feedback Contract
**Apply to:** `.key-touch-target` CSS class, `.bottom-sheet-handle`, `.stack-panel-toggle`

```css
touch-action: manipulation;
-webkit-tap-highlight-color: transparent;
```

---

## No Analog Found

| File | Role | Data Flow | Reason |
|---|---|---|---|
| `hp41-gui/src/scale.ts` (VERIFY ONLY) | utility | transform | No change needed per UI-SPEC — `computeScale` + `ResizeObserver` already handle safe-area shrinkage for free |

`scale.ts` and `main.tsx` ScaledApp are listed in the UI-SPEC touch points but require NO code changes. The `ResizeObserver` pattern already fires when safe-area padding shrinks the content node.

---

## Metadata

**Analog search scope:** `hp41-gui/src/`, `hp41-gui/src-tauri/src/`, `hp41-gui/src-tauri/permissions/`, `hp41-gui/src-tauri/capabilities/`, `hp41-gui/src-tauri/gen/apple/`
**Files read:** 20 source files
**Pattern extraction date:** 2026-06-03
