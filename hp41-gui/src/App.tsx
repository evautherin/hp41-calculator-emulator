import { useState, useEffect, useCallback, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import './App.css';
import { Keyboard, KEY_DEFS, THEME_GRADIENTS, type KeyDef } from './Keyboard';
import { triggerHaptic, maybeFireErrorHaptic, ensureAudioResumed } from './haptics';
import Display14Seg from './Display14Seg';
import HelpOverlay from './HelpOverlay';
import SettingsPanel from './SettingsPanel';
import OnboardingWizard from './OnboardingWizard';
import RawPickerOverlay from './RawPickerOverlay';
import AlphaTouchInput from './AlphaTouchInput';
import BottomSheet from './BottomSheet';
import {
  handleModalKey,
  renderModalLcd,
  makeKeyCodeMagic,
  SUBMIT_MODAL_WITH_LABEL_PREFIX,
  type PendingInput,
  type ModalKeyResult,
} from './pending_input';

// D-50.4/D-50.5: multi-program entry from the backend picker response
interface ProgramEntry {
  label: string;
  index: number;
  byte_len: number;
}

// D-50.4: picker state — set when import_raw_dialog returns a multi-program archive
interface PickerData {
  programs: ProgramEntry[];
  filePath: string;
}

interface Annunciators {
  user: boolean;
  prgm: boolean;
  alpha: boolean;
  rad: boolean;
  grad: boolean;
}

interface CalcStateView {
  display_str: string;
  x_str: string;
  y_str: string;
  z_str: string;
  t_str: string;
  lastx_str: string;
  in_eex_mode: boolean;
  annunciators: Annunciators;
  print_lines: string[];
  program_steps: string[];  // Phase 18 D-01: pre-formatted step strings from Rust
  pc: number;               // Phase 18 D-01: current program counter index
  // Phase 26 D-26.11 (BLOCKER B5): TS-side mirror of the new Rust projections.
  user_keymap: Array<[number, string]>;   // mirrors Vec<(u8, String)>
  flags: number[];                         // mirrors Vec<u8> of set-flag indices
  display_override: string | null;         // mirrors Option<String>
  event_buffer: string[];                  // mirrors Vec<String> (drained per IPC)
  // Phase 31 Plan 03: modal workflow state for D-31.1 / D-31.2 dispatch routing.
  is_running: boolean;                     // mirrors CalcState.is_running
  modal_program_active: boolean;           // mirrors state.modal_program.is_some()
  modal_requires_alpha_label: boolean;     // true when FUNCTION NAME? prompt step is active
  modal_prompt: string | null;             // mirrors Option<String> (null when no modal)
  // Phase 41 D-41.3: live-display trigger fields (mirrors CalcState transient booleans).
  // Frontend starts setInterval(100ms) when either is true; clears when both are false (D-41.8).
  clock_active: boolean;
  stopwatch_keyboard_mode: boolean;
  stopwatch_running: boolean;
}

// Tauri rejects with GuiError { message: string } — String(err) yields
// "[object Object]". Extract the message field so toasts are readable.
// For other object-shaped rejections (raw Tauri framework errors, third-
// party rejections without `.message`) fall back to JSON.stringify so the
// "[object Object]" failure mode this helper exists to prevent cannot leak
// through. Strings, numbers, null, undefined fall through to String().
function extractErrMessage(err: unknown): string {
  if (typeof err === 'object' && err !== null) {
    if ('message' in err) return String((err as { message: unknown }).message);
    try {
      return JSON.stringify(err);
    } catch {
      // Circular reference or non-serialisable value — fall through to String.
    }
  }
  return String(err);
}

// Route a resolved op id to the right Tauri command. SST/BST/R-S have
// dedicated commands; everything else flows through dispatch_op.
//
// Phase 31 Plan 05:
// - Extended with `state: CalcStateView | null` parameter for R/S 3-way routing.
// - Magic-prefix route: `__submit_modal_with_label__<label>` → submit_modal_with_label
// - R/S 3-way (D-31.1): modal_program_active → submit_modal; is_running → request_cancel
//   + get_state; else → existing run_stop.
async function invokeForKey(
  effectiveId: string,
  state: CalcStateView | null,
): Promise<CalcStateView> {
  // Magic-prefix: CollectForModal Enter dispatches __submit_modal_with_label__<label>
  // (Pitfall 15 — must check BEFORE the 'r_s' branch)
  if (effectiveId.startsWith(SUBMIT_MODAL_WITH_LABEL_PREFIX)) {
    const label = effectiveId.slice(SUBMIT_MODAL_WITH_LABEL_PREFIX.length);
    return invoke<CalcStateView>('submit_modal_with_label', { label });
  }
  if (effectiveId === 'sst') return invoke<CalcStateView>('sst_step');
  if (effectiveId === 'bst') return invoke<CalcStateView>('bst_step');
  if (effectiveId === 'r_s') {
    // D-31.1 R/S 3-way state-routed dispatch:
    //   1. modal_program_active → submit_modal (advances the modal step)
    //   2. is_running → request_cancel + get_state (cancels long-running op)
    //   3. else → existing run_stop (R/S key toggle)
    if (state?.modal_program_active) {
      return invoke<CalcStateView>('submit_modal');
    }
    if (state?.is_running) {
      await invoke<void>('request_cancel');
      return invoke<CalcStateView>('get_state');
    }
    return invoke<CalcStateView>('run_stop');
  }
  return invoke<CalcStateView>('dispatch_op', { keyId: effectiveId });
}

function resolveKeyId(e: KeyboardEvent, state: CalcStateView | null): string | null {
  // Phase 49 D-49.12/D-49.13 — Ctrl+key / Cmd+key bindings (T-49-07 mitigation).
  // MUST come BEFORE the F7/F8 checks and the letter MAP to prevent Ctrl+W/R/D/F/S
  // from falling through to the letter map (D-07: never silently discard).
  // metaKey = macOS Cmd (mirrors the Ctrl behavior per RESEARCH A1).
  if (e.ctrlKey || e.metaKey) {
    switch (e.key.toLowerCase()) {
      case 'w': return 'xeq_WPRGM';   // KBD-01: card reader write program
      case 'r': return 'xeq_RDPRGM';  // KBD-01: card reader read program
      case 'd': return 'xeq_WDTA';    // KBD-01: card reader write data
      case 'f': return 'xeq_RDTA';    // KBD-01: card reader read data
      case 's': return '__save_state__'; // KBD-02: manual save
      default: return null; // other Ctrl/Cmd combos — do NOT fall through to letter map
    }
  }
  // Phase 49 KBD-02 — F5: manual save (GUI-only deliberate divergence from CLI).
  // CLI F5 = run_program("A"). GUI F5 = save (D-49.13). e.preventDefault() called
  // in handleKey to prevent the Tauri WebView from reloading (T-49-09).
  if (e.key === 'F5') return '__save_state__';
  // Phase 18 D-07: F7/F8 → SST/BST keyboard bindings
  // Use e.code (physical key) so macOS media-key remapping doesn't block these
  if (e.key === 'F7' || e.code === 'F7') return 'sst';
  if (e.key === 'F8' || e.code === 'F8') return 'bst';

  // ALPHA-mode pass-through: when the ALPHA annunciator is active, route
  // single printable A-Z / 0-9 / space keys to alpha_<X> so the backend
  // resolves them as Op::AlphaAppend.  This must come BEFORE the normal key
  // map so typing "QUAD" in ALPHA mode appends to the alpha register rather
  // than dispatching SIN/UNDEF/ASIN/SDEV.
  if (state?.annunciators?.alpha && e.key.length === 1) {
    const ch = e.key.toUpperCase();
    if (/^[A-Z0-9 ]$/.test(ch)) {
      return `alpha_${ch}`;
    }
  }

  // EEX-CHS: 'n' routes based on current in_eex_mode (D-06)
  if (e.key === 'n') return state?.in_eex_mode ? 'eex_chs' : 'chs';
  // Digit entry
  if (e.key.length === 1 && '0123456789'.includes(e.key)) return e.key;
  if (e.key === '.') return '.';
  if (e.key === 'e') return 'e';
  // Modal-trigger keys — silently ignore, no invoke (D-05).
  // 'P' was in this list pre-Phase-26 but Phase 26 D-26.10 reassigns it
  // to 'prx' (SHIFT+P prints X). 'p' (lowercase) was 'prx' pre-Phase-26
  // and is now 'prgm_mode' per D-26.10.
  if (e.key.length === 1 && 'SRfFX'.includes(e.key)) return null;
  // Named op mapping — authoritative source: hp41-cli/src/keys.rs key_to_op()
  const MAP: Record<string, string> = {
    // 'Backspace' → 'entry_backspace' (not 'clx'): the shared backspace_entry()
    // core helper gives per-digit deletion during entry, CLX when buf is empty.
    // This is the HP-41 fidelity fix — matches CLI behaviour (D-25.6).
    'Enter': 'enter', 'Backspace': 'entry_backspace',
    '+': 'plus', '-': 'minus', '*': 'mul', '/': 'div',
    'r': 'rdn', 'x': 'xy_swap', 'l': 'lastx', 's': 'sqrt',
    // Phase 26 D-26.10 — physical-keyboard 'p' remap.
    // Pre-Phase-26: 'p' was 'prx'. The conflict with the cluster of
    // letter-key shortcuts (every other letter is a math/program op,
    // not a print directive) was deferred from v2.0. v2.2 resolves it:
    // lowercase 'p' now toggles PRGM mode; SHIFT+'P' prints X.
    'p': 'prgm_mode',
    'P': 'prx',
    'a': 'asin', 'c': 'acos', 'k': 'atan',
    'C': 'cos', 'T': 'tan', 'L': 'ln', 'G': 'log', 'E': 'exp',
    'H': 'tenpow', 'I': 'recip', 'W': 'sq', 'Y': 'ypow',
    '%': 'pct_change',
    'u': 'user_mode',
    'z': 'sigma_plus', 'Z': 'sigma_minus', 'm': 'mean', 'D': 'sdev',
    'y': 'yhat', 'b': 'lr', 'O': 'corr', 'V': 'cl_sigma_stat',
    'h': 'hms_to_h', 'j': 'hms_add', 'J': 'hms_sub',
    'q': 'sin',    // Phase 8 reassignment: 'q' = SIN
    'g': 'clreg',  // Phase 8 addition: 'g' = CLREG
  };
  return MAP[e.key] ?? null;
}

// Phase 26 D-26.5 — modal-opener factory table. Mapping from clickable
// modal-opener id → factory function returning the initial PendingInput.
// The 13 *_prompt ids + asn/view/catalog/xeq_prompt/gto_prompt/lbl_prompt are
// intercepted in `handleClick` BEFORE they reach `dispatch_op` — the backend
// stub-error arm in key_map.rs stays as defense-in-depth (D-07 invariant).
//
// The 4 conditional-test prompts (x_eq_y_prompt, etc.) route through the
// `direct` variant (B1) — they dispatch immediately on the next handleModalKey
// call; no accumulator, no IND-toggle.
const MODAL_OPENERS: Record<string, () => PendingInput> = {
  // Register-modal openers (accumulator + IND-toggle).
  sto_prompt: () => ({ kind: 'register', op: 'Sto', ind: false, acc: '' }),
  rcl_prompt: () => ({ kind: 'register', op: 'Rcl', ind: false, acc: '' }),
  isg_prompt: () => ({ kind: 'register', op: 'Isg', ind: false, acc: '' }),
  // Flag-modal openers (accumulator + IND-toggle).
  sf_prompt: () => ({ kind: 'flag', testKind: 'SF', ind: false, acc: '' }),
  cf_prompt: () => ({ kind: 'flag', testKind: 'CF', ind: false, acc: '' }),
  fs_prompt: () => ({ kind: 'flag', testKind: 'FsQuery', ind: false, acc: '' }),
  // Fmt-modal openers (single-digit FIX/SCI/ENG N).
  fix_prompt: () => ({ kind: 'fmt', mode: 'fix' }),
  sci_prompt: () => ({ kind: 'fmt', mode: 'sci' }),
  eng_prompt: () => ({ kind: 'fmt', mode: 'eng' }),
  // BLOCKER B1: conditional-test prompts dispatch IMMEDIATELY via `direct`.
  x_eq_y_prompt: () => ({ kind: 'direct', dispatchId: 'x_eq_y' }),
  x_le_y_prompt: () => ({ kind: 'direct', dispatchId: 'x_le_y' }),
  x_gt_y_prompt: () => ({ kind: 'direct', dispatchId: 'x_gt_y' }),
  x_eq_0_prompt: () => ({ kind: 'direct', dispatchId: 'x_eq_0' }),
  // Label-bearing modals (text input + Enter) — xeq_name shape reused.
  xeq_prompt: () => ({ kind: 'xeq_name', acc: '', dispatchPrefix: 'xeq' }),
  gto_prompt: () => ({ kind: 'xeq_name', acc: '', dispatchPrefix: 'gto' }),
  lbl_prompt: () => ({ kind: 'xeq_name', acc: '', dispatchPrefix: 'lbl' }),
  // v2.2.1 / quick-task 260516-c1p — CLP modal opener. Backend (Op::Clp +
  // key_map.rs resolve clp_<name>) and modal shape (pending_input.ts kind:
  // 'clp') were ready in v2.2 but no opener was wired. Reached via SHIFT +
  // √x in PRGM mode (mode-aware shiftedInPrgm on KeyDef).
  clp_prompt: () => ({ kind: 'clp', acc: '' }),
  // BLOCKER B2: catalog + tone share single_digit with op + max discriminator.
  // Phase 26 Plan 04 CR-05 — Catalog max raised from 3 to 4 so XFNS (CAT 4) is
  // reachable from the GUI; matches hp41-core op_catalog (accepts n in 1..=4).
  catalog: () => ({ kind: 'single_digit', op: 'Catalog', max: 4 }),
  tone: () => ({ kind: 'single_digit', op: 'Tone', max: 9 }),
  // ASN flow: AssignKey → (next key click via __keycode__NN) → AssignLabel.
  asn: () => ({ kind: 'assign_key' }),
  // VIEW — takes a register, same shape as register-modal but op='View'.
  view: () => ({ kind: 'register', op: 'View', ind: false, acc: '' }),
};

function App() {
  const [calcState, setCalcState] = useState<CalcStateView | null>(null);
  // Surfaces GuiError messages from the backend (DivByZero, "unknown key", load
  // failure, lock poison, …). Without this row every HpError translated via
  // From<HpError> ends up at console.error and the user sees stale state.
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const busyRef = useRef(false);
  // Phase 41 D-41.2/D-41.8: live-display interval reference.
  // Holds the setInterval ID when clock_active || stopwatch_keyboard_mode is true.
  const liveTickRef = useRef<ReturnType<typeof setInterval> | null>(null);
  const [printLog, setPrintLog] = useState<string[]>([]);
  const [printPanelOpen, setPrintPanelOpen] = useState(false);
  const printEndRef = useRef<HTMLDivElement>(null);
  const activeStepRef = useRef<HTMLDivElement>(null);
  // Frontend-owned SHIFT one-shot prefix (no IPC round-trip).
  const [shiftActive, setShiftActive] = useState(false);
  // Phase 26 D-26.1 — frontend-owned modal state (no IPC round-trip).
  // PendingInput | null carries the 14 logical states from CLI Phase 25
  // (parity invariant D-25.6). Accumulator keystrokes route through
  // `handleModalKey`; dispatch happens at end-of-accumulation via
  // `invokeForKey(parameterizedId)`. Esc clears both shiftActive AND
  // pendingInput.
  const [pendingInput, setPendingInput] = useState<PendingInput | null>(null);
  // Phase 26 D-26.8 — `?` help overlay open/close. Frontend-only state
  // (no IPC round-trip — the help data is bundled at build time via
  // help_data.ts's vite JSON import).
  const [helpOpen, setHelpOpen] = useState(false);
  // Phase 48 D-48.1/D-48.13 — settings panel open/close + active theme.
  // Theme defaults to 'dark'; overridden by get_prefs on mount.
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [theme, setTheme] = useState<string>('dark');
  // macOS launch mode — overridden by get_prefs on mount; only meaningful on macOS.
  const [macosLaunchMode, setMacosLaunchMode] = useState<string>('menu-bar');
  const [isMacos, setIsMacos] = useState(false);
  const [isIos, setIsIos] = useState(false); // D-55.1 — gates touch behaviors on iOS
  // Phase 55 Plan 05 — iOS collapsible stack panel (TOUCH-10).
  // Default collapsed on iOS to give the keypad more vertical room.
  // Local state only — not persisted to GuiPrefs (resets to collapsed on each launch).
  // On desktop (isIos=false) this state is unused; the stack stays fully expanded.
  const [stackExpanded, setStackExpanded] = useState(false);
  // Phase 49 D-49.4/D-49.8/ONBOARD-01 — onboarding wizard overlay state.
  // onboardingOpen: wizard visible; isFirstRun: true when auto-opened on first launch
  //   (Esc blocked in first-run mode per D-49.9), false when re-opened from settings.
  const [onboardingOpen, setOnboardingOpen] = useState(false);
  const [isFirstRun, setIsFirstRun] = useState(false);
  // Phase 50 D-50.4/D-50.6 — multi-program picker overlay state.
  // null when picker is closed; set when import_raw_dialog returns a multi-program archive.
  const [pickerData, setPickerData] = useState<PickerData | null>(null);
  // Toast overlay for GuiError responses (single-toast policy, 2s auto-dismiss).
  // The monotonic `seq` is required because two clicks on the same stubbed
  // key produce identical message strings — setting state to the same value
  // does not re-run the auto-dismiss effect, so the second toast would be
  // dismissed by the first click's still-running timer. `seq` makes the
  // state value distinct on each call.
  const [toast, setToast] = useState<{ msg: string; seq: number } | null>(null);
  const toastSeqRef = useRef(0);
  // Phase 55 Plan 03: iOS haptic + audio refs.
  // audioResumedRef: one-shot guard — AudioContext resumed once on first pointerdown (TOUCH-06).
  // errorHapticFiredRef: prevents notificationFeedback('error') from re-firing on every render
  //   while a DATA ERROR / NO ROOM display is active (T-55-06 / Pitfall 7).
  // audioCtxRef: holds the AudioContext instance created lazily on first iOS gesture;
  //   TONE/BEEP audio is currently routed through Rust but Web Audio is pre-unocked here
  //   so future audio additions are immediately audible without a gesture barrier.
  const audioResumedRef = useRef(false);
  const errorHapticFiredRef = useRef(false);
  const audioCtxRef = useRef<AudioContext | null>(null);
  const showToast = useCallback((msg: string) => {
    toastSeqRef.current += 1;
    setToast({ msg, seq: toastSeqRef.current });
  }, []);

  // Auto-dismiss toast after 2 seconds. Re-runs on every showToast() call
  // because the `seq` field changes even when `msg` is the same.
  useEffect(() => {
    if (!toast) return;
    const t = setTimeout(() => setToast(null), 2000);
    return () => clearTimeout(t);
  }, [toast]);

  // Phase 48 D-48.13 — theme change handler.
  // Applies theme instantly via CSS variable system + gradient prop, then
  // persists to prefs.json via fire-and-forget IPC (D-48.13: errors silently logged).
  const handleThemeChange = useCallback((newTheme: string) => {
    setTheme(newTheme);
    document.body.dataset.theme = newTheme;
    invoke('set_pref', { key: 'theme', value: newTheme }).catch(() => {
      // Persistence failure is non-fatal — theme applies visually regardless.
    });
  }, []);

  // macOS launch-mode change — persist via fire-and-forget IPC (D-48.13 pattern).
  // The new mode is applied at next startup (decided in setup()), so SettingsPanel
  // reveals a "Restart now" affordance after the change.
  const handleLaunchModeChange = useCallback((mode: string) => {
    setMacosLaunchMode(mode);
    invoke('set_pref', { key: 'macos_launch_mode', value: mode }).catch(() => {
      // Persistence failure is non-fatal — the choice is re-applied on next change.
    });
  }, []);

  // Phase 49 ONBOARD-01/ONBOARD-05 — close handler for the onboarding wizard.
  // Marks onboarding_done=true via fire-and-forget IPC (D-48.13 pattern).
  // Value passed as string "true" per RESEARCH Pitfall 3 (Tauri IPC boolean encoding).
  const handleOnboardingClose = useCallback(() => {
    setOnboardingOpen(false);
    setIsFirstRun(false);
    invoke('set_pref', { key: 'onboarding_done', value: 'true' }).catch(() => {
      // Persistence failure is non-fatal — wizard closes regardless.
    });
  }, []);

  // Phase 49 D-49.8/D-49.9 — re-open wizard from Settings panel.
  // Closes settings first, then opens wizard in re-open mode (isFirstRun=false).
  // isFirstRun=false allows Esc to close the wizard (not first-run — D-49.9).
  const handleShowOnboarding = useCallback(() => {
    setSettingsOpen(false);
    setIsFirstRun(false);  // re-open mode: Esc allowed to close wizard
    setOnboardingOpen(true);
  }, []);

  // Phase 50 D-50.1/D-50.3/D-50.4 — file dialog functions for card reader ops with empty ALPHA.
  // All dialog functions guard with busyRef. The Tauri backend opens the native OS dialog,
  // so the frontend does NOT import @tauri-apps/plugin-dialog JS API directly.

  // importRawDialog: opens OS file picker for .raw import.
  // Single-program response: setCalcState + toast per D-50.3.
  // Multi-program response: setPickerData to open the picker overlay per D-50.4.
  const importRawDialog = useCallback(async () => {
    if (busyRef.current) return;
    busyRef.current = true;
    try {
      const resp = await invoke<{
        type: string;
        view?: CalcStateView;
        message?: string;
        programs?: ProgramEntry[];
        file_path?: string;
      }>('import_raw_dialog');
      if (resp.type === 'Single' && resp.view && resp.message) {
        setCalcState(resp.view);
        setErrorMessage(null);
        showToast(resp.message);
      } else if (resp.type === 'Multi' && resp.programs && resp.file_path) {
        // Multi-program: open picker; busyRef released so picker can dispatch
        setPickerData({ programs: resp.programs, filePath: resp.file_path });
      } else if (resp.type === 'Empty') {
        showToast('No programs found in file');
      }
      // Cancelled: no feedback (silent per UI-SPEC)
    } catch (err) {
      showToast(extractErrMessage(err));
    } finally {
      busyRef.current = false;
    }
  }, [showToast]);

  // exportRawDialog: opens OS save dialog for .raw export.
  const exportRawDialog = useCallback(async () => {
    if (busyRef.current) return;
    busyRef.current = true;
    try {
      const resp = await invoke<{ message?: string; cancelled?: boolean }>('export_raw_dialog');
      if (resp.message) {
        showToast(resp.message);
      }
      // cancelled: silent no-op per UI-SPEC
    } catch (err) {
      showToast(extractErrMessage(err));
    } finally {
      busyRef.current = false;
    }
  }, [showToast]);

  // importDataDialog: opens OS file picker for .card.json data card import (D-50.7).
  const importDataDialog = useCallback(async () => {
    if (busyRef.current) return;
    busyRef.current = true;
    try {
      const resp = await invoke<{ view: CalcStateView; message: string }>('import_data_dialog');
      setCalcState(resp.view);
      setErrorMessage(null);
      showToast(resp.message);
    } catch (err) {
      showToast(extractErrMessage(err));
    } finally {
      busyRef.current = false;
    }
  }, [showToast]);

  // exportDataDialog: opens OS save dialog for .card.json data card export (D-50.7).
  const exportDataDialog = useCallback(async () => {
    if (busyRef.current) return;
    busyRef.current = true;
    try {
      const resp = await invoke<{ message?: string; cancelled?: boolean }>('export_data_dialog');
      if (resp.message) {
        showToast(resp.message);
      }
      // cancelled: silent no-op per UI-SPEC
    } catch (err) {
      showToast(extractErrMessage(err));
    } finally {
      busyRef.current = false;
    }
  }, [showToast]);

  // handlePickerConfirm: imports selected programs from the multi-program archive (D-50.6).
  const handlePickerConfirm = useCallback(async (selectedIndices: number[]) => {
    if (!pickerData) return;
    if (busyRef.current) return;
    busyRef.current = true;
    try {
      const result = await invoke<CalcStateView>('import_selected_programs', {
        filePath: pickerData.filePath,
        indices: selectedIndices,
      });
      setCalcState(result);
      setErrorMessage(null);
      showToast(`Imported ${selectedIndices.length} program${selectedIndices.length === 1 ? '' : 's'}`);
      setPickerData(null);
    } catch (err) {
      showToast(extractErrMessage(err));
    } finally {
      busyRef.current = false;
    }
  }, [pickerData, showToast]);

  // handlePickerClose: dismisses the picker without importing (UI-SPEC: silent dismiss).
  const handlePickerClose = useCallback(() => {
    setPickerData(null);
  }, []);

  // Phase 41 D-41.2/D-41.3/D-41.8: start/stop the 100ms live-display interval.
  //
  // Starts when calcState.clock_active || calcState.stopwatch_keyboard_mode is true.
  // CR-01 fix: derive needsTick as a stable boolean so the interval useEffect
  // does not depend on the full calcState object (which changes every tick).
  const needsTick = Boolean(calcState?.clock_active || calcState?.stopwatch_keyboard_mode);

  useEffect(() => {
    if (needsTick && liveTickRef.current === null) {
      liveTickRef.current = setInterval(() => {
        if (busyRef.current) return;
        invoke<CalcStateView>('tick_time')
          .then(view => { setCalcState(view); setErrorMessage(null); })
          .catch(err => showToast(extractErrMessage(err)));
      }, 100);
    } else if (!needsTick && liveTickRef.current !== null) {
      clearInterval(liveTickRef.current);
      liveTickRef.current = null;
    }
    return () => {
      if (liveTickRef.current !== null) {
        clearInterval(liveTickRef.current);
        liveTickRef.current = null;
      }
    };
  }, [needsTick, showToast]);

  // Mount: load initial state via get_state (D-11 — no polling)
  useEffect(() => {
    invoke<CalcStateView>('get_state')
      .then(view => { setCalcState(view); setErrorMessage(null); })
      .catch(err => setErrorMessage(`Load failed: ${err}`));
  }, []);

  useEffect(() => {
    invoke<boolean>('is_macos').then(setIsMacos).catch(() => setIsMacos(false));
  }, []);

  // D-55.1 — detect iOS to gate touch behaviors (bottom sheets, collapsible stack,
  // AlphaTouchInput bar, .key-touch-target overlays, haptic calls).
  useEffect(() => {
    invoke<boolean>('is_ios').then(setIsIos).catch(() => setIsIos(false));
  }, []);

  // Phase 48 D-48.13 + Phase 49 ONBOARD-01/ONBOARD-05 — load persisted preferences on mount.
  // Sets document.body.dataset.theme to drive themes.css [data-theme] blocks.
  // Checks onboarding_done to auto-open wizard on first run (P59: lives in prefs.json,
  // NEVER in autosave.json). Silently falls back to 'dark' and opens wizard if prefs.json
  // is missing (first-run fallback — D-49.4 / RESEARCH Pitfall 3).
  useEffect(() => {
    invoke<{ theme: string; onboarding_done: boolean; macos_launch_mode: string }>('get_prefs')
      .then(prefs => {
        setTheme(prefs.theme);
        document.body.dataset.theme = prefs.theme;
        setMacosLaunchMode(prefs.macos_launch_mode);
        if (!prefs.onboarding_done) {
          // First launch: auto-open wizard in first-run mode (Esc blocked per D-49.9).
          setIsFirstRun(true);
          setOnboardingOpen(true);
        }
      })
      .catch(() => {
        document.body.dataset.theme = 'dark';
        // prefs.json missing → first run. Open wizard in first-run mode.
        setIsFirstRun(true);
        setOnboardingOpen(true);
      });
  }, []);

  // Physical-keyboard dispatch (option B): string-id path, no SHIFT/ALPHA frontend
  // mediation. resolveKeyId already maps physical keys to op ids directly. SST/BST
  // route to their dedicated Tauri commands; everything else goes through dispatch_op.
  // Errors surface as a toast (consistent with on-screen keyboard).
  const dispatchKeyId = useCallback((keyId: string) => {
    if (busyRef.current) return;
    busyRef.current = true;
    invokeForKey(keyId, calcState)
      .then(view => {
        setCalcState(view);
        setErrorMessage(null);
        void maybeFireErrorHaptic(view.display_str, isIos, errorHapticFiredRef);
      })
      .catch(err => showToast(extractErrMessage(err)))
      .finally(() => { busyRef.current = false; });
  }, [calcState, isIos, showToast]);

  // Apply a ModalKeyResult — updates state and optionally dispatches.
  // Returns true if a dispatch was issued (caller can short-circuit).
  const applyModalResult = useCallback(
    async (result: ModalKeyResult): Promise<boolean> => {
      setPendingInput(result.nextPending);
      if (result.consumesShift) setShiftActive(false);
      if (result.dispatchId === null) return false;
      busyRef.current = true;
      try {
        const view = await invokeForKey(result.dispatchId, calcState);
        setCalcState(view);
        setErrorMessage(null);
        void maybeFireErrorHaptic(view.display_str, isIos, errorHapticFiredRef);
      } catch (err) {
        showToast(extractErrMessage(err));
      } finally {
        busyRef.current = false;
      }
      return true;
    },
    [calcState, isIos, showToast],
  );

  // On-screen keyboard click router. Resolution order:
  //   1. SHIFT key toggles local shiftActive, no dispatch.
  //   2. ALPHA mode + alphaChar → alpha_<char> (SHIFT ignored in ALPHA mode).
  //   3. shiftActive + key.shifted → shifted.id, consumes the one-shot.
  //   4. otherwise → primary id.
  //   5. Phase 26 D-26.5: if effectiveId in MODAL_OPENERS, intercept BEFORE
  //      invokeForKey and open the React modal. The `direct` variant resolves
  //      on the same tick (B1).
  //   6. If a modal is already open (pendingInput !== null), route the click
  //      through handleModalKey (D-26.2).
  // Special routes: sst/bst/r_s go to dedicated commands; clx_or_a branches on
  // the live ALPHA annunciator into clx | alpha_clear.
  const handleClick = useCallback(async (key: KeyDef) => {
    if (busyRef.current) return;

    // SHIFT key itself toggles state, no dispatch (rule 1 above).
    // CRITICAL (W2): SHIFT toggling inside an open modal goes HERE, NOT
    // through handleModalKey — the modal's IND-toggle path requires
    // shiftActive to be set BEFORE the "0" keystroke arrives.
    if (key.id === 'shift') {
      setShiftActive(prev => !prev);
      return;
    }

    // Resolve the effective id per rules 2-4.
    const alphaOn = calcState?.annunciators.alpha ?? false;
    const prgmOn = calcState?.annunciators.prgm ?? false;
    let effectiveId: string;
    let consumesShift = false;

    if (alphaOn && key.alphaChar) {
      effectiveId = `alpha_${key.alphaChar}`;
    } else if (shiftActive && (key.shifted || key.shiftedInPrgm)) {
      // v2.2.1 / quick-task 260516-c1p — mode-aware shifted variant.
      // When PRGM is active AND the key carries shiftedInPrgm, prefer
      // that id (e.g. √x → CLP in PRGM; → x² outside). The fall-through
      // to `key.id` only triggers for a future shiftedInPrgm-only entry
      // pressed outside PRGM; today's only such entry (sqrt) also has a
      // regular shifted slot, so the branch stays unreachable for now.
      if (prgmOn && key.shiftedInPrgm) {
        effectiveId = key.shiftedInPrgm.id;
      } else if (key.shifted) {
        effectiveId = key.shifted.id;
      } else {
        effectiveId = key.id;
      }
      consumesShift = true;
    } else {
      effectiveId = key.id;
    }

    if (!effectiveId) return;

    // Rule 6: if a modal is open, route through handleModalKey.
    if (pendingInput !== null) {
      // Phase 26 Plan 04 — translate on-screen click ids to the modal's
      // key alphabet, then route through handleModalKey. Cases in priority order:
      //   (a) assign_key + key.keyCode defined: encode the canonical HP-41
      //       hardware keyCode via makeKeyCodeMagic. CR-01: use key.keyCode
      //       (hardcoded CLI-canonical literal per Keyboard.tsx W9 doc),
      //       NOT a computed row-times-ten formula on (row, col) which
      //       would store the ASN at a keyCode no KeyDef advertises
      //       (breaking USER-mode relabel end-to-end).
      //   (b) assign_key + key.keyCode undefined: the clicked key has no
      //       canonical HP-41 mapping (variant 'top'/'shift', CHS, xge_y,
      //       clx_or_a, empty-id). Surface a toast and leave the modal
      //       open. D-07 forbids silent discards.
      //   (c) TOUCH-04 (Phase 55 Plan 06 supersede) — text-label modal
      //       (xeq_name, clp, assign_label) + effectiveId === 'alpha_toggle':
      //       maps to 'Enter' so the on-screen ALPHA key terminates/submits.
      //       HP-41-faithful: the ALPHA key exits ALPHA-entry mode on hardware.
      //   (d) TOUCH-04 — text-label modal + key.alphaChar defined: route the
      //       alphaChar (single uppercase letter) instead of the op-id.
      //       MUST come before (e) so on-screen ENTER (alphaChar='N') types 'N'
      //       rather than terminating. Also routes Σ+(A), 1/x(B), √x(C) etc.
      //       NOTE: physical keyboard is NOT affected — see handleKey below.
      //       On-screen-vs-physical divergence is accepted (user decision,
      //       TOUCH-04): physical Enter keeps terminating for D-25.6 parity.
      //   (e) effectiveId === 'enter' / 'clx_or_a': translate to 'Enter' /
      //       'Backspace' for non-text-label modals (CR-03 fix; text-label
      //       ENTER is handled in (d) above so this branch covers fmt, flag,
      //       register, etc.). Backspace also applies to text-label kinds.
      //   (f) default: forward effectiveId verbatim.
      const isTextLabelKind =
        pendingInput.kind === 'xeq_name' ||
        pendingInput.kind === 'clp' ||
        pendingInput.kind === 'assign_label';
      let routedKey: string;
      if (pendingInput.kind === 'assign_key') {
        if (key.keyCode === undefined) {
          // CR-01 toast: defense-in-depth surfacing of the W9 contract
          // (variant 'top'/'shift' + chs + xge_y + clx_or_a have no
          // unambiguous CLI mapping — they cannot be ASN targets).
          showToast('This key cannot be assigned');
          if (consumesShift) setShiftActive(false);
          return;
        }
        routedKey = makeKeyCodeMagic(key.keyCode);
      } else if (isTextLabelKind && effectiveId === 'alpha_toggle') {
        // (c) TOUCH-04: on-screen ALPHA terminates text-label modals.
        routedKey = 'Enter';
      } else if (isTextLabelKind && key.alphaChar) {
        // (d) TOUCH-04: on-screen keys with alphaChar type their letter.
        // ENTER (alphaChar='N') types 'N'; priority over branch (e) below.
        routedKey = key.alphaChar;
      } else if (effectiveId === 'enter') {
        routedKey = 'Enter';
      } else if (effectiveId === 'clx_or_a') {
        routedKey = 'Backspace';
      } else {
        routedKey = effectiveId;
      }
      const result = handleModalKey(routedKey, pendingInput, shiftActive);
      // Consume the click shift on a modal-key transition too.
      if (consumesShift && !result.consumesShift) setShiftActive(false);
      await applyModalResult(result);
      return;
    }

    // Rule 5: modal-opener intercept (D-26.5).
    if (MODAL_OPENERS[effectiveId]) {
      const initial = MODAL_OPENERS[effectiveId]();
      // B1 fast path: `direct` variant — open and resolve on the same tick.
      if (initial.kind === 'direct') {
        const result = handleModalKey('', initial, false);
        if (consumesShift) setShiftActive(false);
        await applyModalResult(result);
        return;
      }
      setPendingInput(initial);
      if (consumesShift) setShiftActive(false);
      return;
    }

    busyRef.current = true;
    try {
      let view: CalcStateView;
      if (effectiveId === 'clx_or_a') {
        // CL X/A — branch on alpha mode at click time. (On-screen-specific:
        // physical-keyboard has no equivalent path, so this stays out of
        // invokeForKey and lives here in handleClick.)
        // Non-alpha: 'entry_backspace' → backspace_entry() core helper gives
        // per-digit deletion during entry, CLX when no active entry (HP-41
        // fidelity fix, D-25.6). Matches CLI behaviour and physical-keyboard
        // resolveKeyId path. Alpha branch ('alpha_clear') is unchanged.
        const targetId = alphaOn ? 'alpha_clear' : 'entry_backspace';
        view = await invoke<CalcStateView>('dispatch_op', { keyId: targetId });
      } else {
        view = await invokeForKey(effectiveId, calcState);
      }
      setCalcState(view);
      setErrorMessage(null);
      void maybeFireErrorHaptic(view.display_str, isIos, errorHapticFiredRef);
    } catch (err) {
      showToast(extractErrMessage(err));
    } finally {
      if (consumesShift) setShiftActive(false);
      busyRef.current = false;
    }
  }, [calcState, isIos, shiftActive, pendingInput, applyModalResult, showToast]);

  // Physical-keyboard handler — useCallback with calcState dep so 'n' reads latest in_eex_mode.
  // Tab toggles SHIFT, Esc cancels in precedence order: help → modal → shift
  // (Phase 26 D-26.8 + D-26.4).
  const handleKey = useCallback((e: KeyboardEvent) => {
    if (e.repeat) return;        // SC-4 fix: ignore OS key-repeat events — each IPC round-trip
                                 // completes before the next repeat fires, defeating busyRef alone

    // Phase 26 D-26.8 — '?' opens the help overlay. Guard against ALPHA mode
    // (where '?' is a valid ALPHA register input — same convention as the
    // CLI Phase 25 `?` overlay). Skip if the overlay is already open so
    // typing '?' in the search input doesn't re-fire the toggle.
    const alphaOn = calcState?.annunciators.alpha ?? false;
    if (e.key === '?' && !alphaOn && !helpOpen) {
      e.preventDefault();
      setHelpOpen(true);
      setSettingsOpen(false);    // Phase 48: close settings when help opens
      setOnboardingOpen(false);  // Phase 49: mutual exclusion — close wizard when help opens
      return;
    }

    // Esc precedence (D-26.8 + D-26.4 + D-31.2 + D-39.3 + D-39.4 + D-49.9):
    //   -1. Clock display (D-39.3: any key exits — clear and fall through).
    //   0. Stopwatch keyboard mode (exits sw mode via sw_exit dispatch).
    //   1. Onboarding wizard in re-open mode (D-49.9: Esc closes; first-run blocks Esc).
    //   2. Settings overlay (closes on Esc).
    //   3. Help overlay (closes on Esc; doesn't clear modal/shift).
    //   4. pendingInput (closes the modal, clears shiftActive).
    //   5. modal_program_active → cancel_modal.
    //   6. is_running → request_cancel.
    //   7. shiftActive last (clears the one-shot SHIFT prefix).
    if (e.key === 'Escape') {
      if (calcState?.clock_active || calcState?.stopwatch_keyboard_mode) {
        if (!busyRef.current) {
          busyRef.current = true;
          invoke<CalcStateView>('dispatch_op', { keyId: 'sw_exit' })
            .then(view => { setCalcState(view); setErrorMessage(null); })
            .catch(err => showToast(extractErrMessage(err)))
            .finally(() => { busyRef.current = false; });
        }
        if (calcState?.stopwatch_keyboard_mode) return;
      }
      // D-49.9: Esc closes wizard in re-open mode; first-run mode blocks Esc
      // (wizard handles first-run Esc internally by ignoring it).
      // The OnboardingWizard component also has its own Esc handler; this branch
      // provides the App-level mutual-exclusion guarantee.
      if (onboardingOpen && !isFirstRun) {
        handleOnboardingClose();
        return;
      }
      if (settingsOpen) {
        setSettingsOpen(false);
        return;
      }
      if (helpOpen) {
        setHelpOpen(false);
        return;
      }
      if (pendingInput !== null) {
        setPendingInput(null);
        setShiftActive(false);
        return;
      }
      // D-31.2 branch 1: cancel active modal workflow
      if (calcState?.modal_program_active) {
        if (!busyRef.current) {
          busyRef.current = true;
          invoke<CalcStateView>('cancel_modal')
            .then(view => { setCalcState(view); setErrorMessage(null); })
            .catch(err => showToast(extractErrMessage(err)))
            .finally(() => { busyRef.current = false; });
        }
        return;
      }
      // D-31.2 branch 2: cancel long-running op
      if (calcState?.is_running) {
        if (!busyRef.current) {
          busyRef.current = true;
          invoke<void>('request_cancel')
            .then(() => invoke<CalcStateView>('get_state'))
            .then(view => { setCalcState(view); setErrorMessage(null); })
            .catch(err => showToast(extractErrMessage(err)))
            .finally(() => { busyRef.current = false; });
        }
        return;
      }
      // D-31.2 branch 3: clear SHIFT one-shot
      setShiftActive(false);
      return;
    }
    if (e.key === 'Tab') {
      e.preventDefault();
      setShiftActive(prev => !prev);
      return;
    }
    // Phase 26 Plan 04 CR-02 — when the `?` help overlay is open, its
    // <input> search box owns focus. The window-level keydown listener
    // must NOT leak keystrokes to resolveKeyId / dispatchKeyId, or every
    // character typed into the search box also dispatches an Op (e.g.
    // 's' → Op::Sqrt, 'q' → Op::Sin, Backspace → Op::Clx) and corrupts
    // calculator state in the background. Esc and '?' are already
    // handled above; this is the third gate layer.
    if (helpOpen) return;
    // Phase 49 D-49.9 — when the onboarding wizard is open, block keyboard
    // dispatch to the calculator. The wizard owns keyboard events while visible.
    // (Esc is handled above in the Esc precedence block per D-49.9.)
    if (onboardingOpen) return;

    // D-39.4/D-39.5 mirror: stopwatch keyboard mode intercepts all keys.
    // Space/Enter → RUNSW/STOPSW toggle, 's' → split, 'r' → reset, Esc → exit.
    // Key IDs use xeq_ prefix for XROM resolution (key_map.rs xeq_ path).
    if (calcState?.stopwatch_keyboard_mode) {
      e.preventDefault();
      if (busyRef.current) return;
      let swKeyId: string | null = null;
      if (e.key === ' ' || e.key === 'Enter') {
        swKeyId = calcState.stopwatch_running ? 'xeq_STOPSW' : 'xeq_RUNSW';
      } else if (e.key === 's') {
        swKeyId = 'xeq_SWPT';
      } else if (e.key === 'r') {
        swKeyId = 'xeq_STPW';
      }
      if (swKeyId) {
        busyRef.current = true;
        invoke<CalcStateView>('dispatch_op', { keyId: swKeyId })
          .then(view => { setCalcState(view); setErrorMessage(null); })
          .catch(err => showToast(extractErrMessage(err)))
          .finally(() => { busyRef.current = false; });
      }
      return; // all keys consumed in stopwatch mode
    }

    if (busyRef.current) return; // debounce: ignore while invoke pending

    // Phase 26 D-26.4: if a modal is open, route the key through handleModalKey
    // BEFORE the normal resolveKeyId path. Esc is already handled above.
    if (pendingInput !== null) {
      // Translate physical keys to the modal's input alphabet.
      let modalKey: string | null = null;
      if (e.key === 'Enter') modalKey = 'Enter';
      else if (e.key === 'Backspace') modalKey = 'Backspace';
      else if (e.key.length === 1) {
        // Single printable character — digit, letter, or punctuation.
        modalKey = e.key;
      }
      if (modalKey === null) return;
      e.preventDefault();
      const result = handleModalKey(modalKey, pendingInput, shiftActive);
      void applyModalResult(result);
      return;
    }

    let keyId = resolveKeyId(e, calcState);
    if (keyId === null) return;  // unmapped or modal-trigger key — silent ignore

    // Phase 49 D-49.13/KBD-02 — intercept __save_state__ BEFORE dispatchKeyId.
    // T-49-08 mitigation: the backend key_map.rs errors on unknown key IDs (D-07),
    // so __save_state__ must NEVER reach dispatch_op. Also calls e.preventDefault()
    // to block browser save-page (T-49-10) and F5 browser reload (T-49-09).
    if (keyId === '__save_state__') {
      e.preventDefault();
      invoke<void>('save_state')
        .then(() => showToast('Saved'))
        .catch(err => showToast(`Save failed: ${extractErrMessage(err)}`));
      return;
    }

    // Phase 50 D-50.1 — card reader key intercept: when ALPHA annunciator is false
    // (no ALPHA content), open the native OS file dialog instead of using ~/.hp41/cards/.
    // When ALPHA is active, fall through to normal dispatch (backend uses alpha register
    // as the file name in cards_dir — existing behavior preserved).
    const alphaAnnOn = calcState?.annunciators.alpha ?? false;
    if (!alphaAnnOn) {
      if (keyId === 'xeq_RDPRGM') { e.preventDefault(); void importRawDialog(); return; }
      if (keyId === 'xeq_WPRGM')  { e.preventDefault(); void exportRawDialog(); return; }
      if (keyId === 'xeq_RDTA')   { e.preventDefault(); void importDataDialog(); return; }
      if (keyId === 'xeq_WDTA')   { e.preventDefault(); void exportDataDialog(); return; }
    }

    // Quick-task 260522-gud — honor `shiftActive` on the physical-keyboard
    // path so Tab + 0 → π (and every other `f`-prefix combo) matches the
    // on-screen-click behavior in `handleClick` (rule 3, line 324). Mirrors
    // CLI `shifted_key_to_op` in hp41-cli/src/app.rs:484. ALPHA pass-through
    // already returned a `alpha_<X>` id inside `resolveKeyId` (line 114-119)
    // so this block only fires for non-alpha keys. PRGM-mode `shiftedInPrgm`
    // wins over `shifted` when both are present, matching handleClick.
    // Consume-on-swap-only (no setShiftActive(false) on no-match) keeps
    // physical and on-screen paths bit-for-bit identical.
    if (shiftActive) {
      const def = KEY_DEFS.find(k => k.id === keyId);
      if (def) {
        const prgmOn = calcState?.annunciators.prgm ?? false;
        const shiftedId = (prgmOn && def.shiftedInPrgm)
          ? def.shiftedInPrgm.id
          : def.shifted?.id;
        if (shiftedId) {
          keyId = shiftedId;
          setShiftActive(false);
        }
      }
    }

    // Modal opener intercept: shifted key IDs like 'fix_prompt', 'sto_prompt',
    // etc. are frontend-only modal openers — they must NEVER reach dispatch_op
    // (the backend errors on unknown key IDs per D-07). Mirrors handleClick
    // rule 5 (line 527). Without this, Tab+1 on the physical keyboard sends
    // 'fix_prompt' to the backend instead of opening the FIX digit modal.
    if (MODAL_OPENERS[keyId]) {
      const initial = MODAL_OPENERS[keyId]();
      if (initial.kind === 'direct') {
        void applyModalResult(handleModalKey('', initial, false));
        return;
      }
      setPendingInput(initial);
      setShiftActive(false);
      return;
    }

    e.preventDefault();
    dispatchKeyId(keyId);
  }, [calcState, dispatchKeyId, pendingInput, shiftActive, applyModalResult, helpOpen, settingsOpen, onboardingOpen, isFirstRun, handleOnboardingClose, showToast, importRawDialog, exportRawDialog, importDataDialog, exportDataDialog]);

  // Register keyboard listener — cleanup required for React StrictMode (D-12)
  useEffect(() => {
    window.addEventListener('keydown', handleKey);
    return () => window.removeEventListener('keydown', handleKey);
  }, [handleKey]);

  // Phase 54 PERSIST-02: save state when the app is backgrounded (iOS resign-active).
  // Fires in WKWebView when the user presses the Home button or switches apps.
  // Fire-and-forget: the 30s auto-save thread (D-54.2a) is the safety net.
  // Empty deps: handler has no dependency on React state — invoke always saves current state.
  // D-54.2c: no page-hide or unload listeners added (redundant, risk double-saves).
  useEffect(() => {
    const handleVisibilityChange = () => {
      if (document.visibilityState === 'hidden') {
        void invoke<void>('save_state').catch((err: unknown) => {
          // Silent failure acceptable: the 30s timer is the safety net (D-54.2a)
          console.warn('background save failed:', extractErrMessage(err));
        });
      }
    };
    document.addEventListener('visibilitychange', handleVisibilityChange);
    return () => document.removeEventListener('visibilitychange', handleVisibilityChange);
  }, []); // empty deps: listener is stable, registered once on mount

  // Accumulate print_lines from each IPC response into local React state.
  // D-09: print_buffer is drained per IPC call; React retains full history.
  // D-07: setPrintPanelOpen(true) auto-shows panel on first print output.
  useEffect(() => {
    if (calcState && calcState.print_lines.length > 0) {
      setPrintLog(prev => [...prev, ...calcState.print_lines]);
      setPrintPanelOpen(true);
    }
  }, [calcState]);

  // Phase 26 Plan 04 CR-04b — consume calcState.event_buffer per IPC response.
  // The backend (Phase 21 ROM ops) pushes BEEP / TONE / etc. strings into
  // CalcState.event_buffer; commands.rs drains the buffer into the projected
  // CalcStateView.event_buffer Vec on every IPC response. Pre-fix, the
  // projection arrived but React never read it — BEEP/TONE were silently
  // dropped. Surface each event line via the toast queue (single-toast
  // policy; the seq counter inside showToast re-fires identical messages).
  // A future v3.x Web Audio API replacement plugs in here without changing
  // the projection contract; the event_buffer schema stays the same.
  //
  // Phase 41 D-41.6: extended to parse alarm event prefixes from hp41-core alarm.rs:
  //   "alarm:message:{text}" → showToast with prefix stripped (shows only alarm text)
  //   "alarm:xeq:{label}"   → invoke dispatch_op xeq_{label} (control alarm XEQ target)
  //   other lines            → showToast as before (BEEP/TONE/etc.)
  useEffect(() => {
    if (calcState && calcState.event_buffer.length > 0) {
      for (const line of calcState.event_buffer) {
        if (line.startsWith('alarm:message:')) {
          // Strip "alarm:message:" prefix — show only the alarm message text.
          showToast(line.slice('alarm:message:'.length));
        } else if (line.startsWith('alarm:xeq:')) {
          const label = line.slice('alarm:xeq:'.length);
          if (busyRef.current) continue;
          busyRef.current = true;
          invoke<CalcStateView>('dispatch_op', { keyId: `xeq_${label}` })
            .then(view => { setCalcState(view); setErrorMessage(null); })
            .catch(err => showToast(extractErrMessage(err)))
            .finally(() => { busyRef.current = false; });
        } else if (line.startsWith('alarm:interrupting:')) {
          // D-38.4: interrupting control alarms deferred — silently ignore.
        } else {
          showToast(line);
        }
      }
    }
  }, [calcState, showToast]);

  // Phase 31 Plan 05 — D-29.9 GUI mirror: post-dispatch auto-open CollectForModal.
  //
  // After every calcState update, check if a modal_program is active AND requires
  // an alpha label AND no pendingInput is already open. If so, automatically open
  // the XeqByName modal in 'collect-for-modal' mode so the user can type the
  // function label name (e.g. for INTG/SOLVE/DIFEQ's FUNCTION NAME? prompt).
  //
  // Verbatim mirror of hp41-cli/src/app.rs::maybe_auto_open_collect_for_modal
  // (lines 1782-1795). Per D-25.6 CLI ↔ GUI parity invariant.
  //
  // Dependencies [calcState, pendingInput]: re-runs on every state update;
  // early-returns prevent infinite re-fires when pendingInput is already set.
  useEffect(() => {
    if (!calcState) return;
    if (pendingInput !== null) return;
    if (!calcState.modal_program_active) return;
    if (!calcState.modal_requires_alpha_label) return;
    // Auto-open XeqByName in collect-for-modal mode.
    setPendingInput({
      kind: 'xeq_name',
      dispatchPrefix: 'xeq',
      acc: '',
      mode: 'collect-for-modal',
    });
  }, [calcState, pendingInput]);

  // Auto-scroll to bottom whenever the print log grows.
  //
  // WR-03: On iOS the print sentinel lives inside a BottomSheet whose
  // `.bottom-sheet-content` is `overflow: hidden` with a 32px peek while
  // collapsed. Calling scrollIntoView there silently no-ops (and can nudge
  // the outer scroll). We do NOT auto-expand the sheet (that would change
  // behavior the user hasn't asked for); we just skip the scroll while the
  // enclosing bottom sheet is collapsed. On desktop there is no `.bottom-sheet`
  // ancestor, so the guard passes through and behavior is unchanged. We also
  // use `block: 'nearest'` to avoid moving the outer scroll position.
  useEffect(() => {
    const node = printEndRef.current;
    if (!node) return;
    const sheet = node.closest('.bottom-sheet');
    if (sheet && !sheet.classList.contains('expanded')) {
      // Collapsed bottom sheet — scrolling is a no-op; skip to avoid the
      // misleading unconditional call and any outer-scroll nudge.
      return;
    }
    node.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
  }, [printLog]);

  // Auto-scroll active program step into view when pc changes (D-09)
  useEffect(() => {
    activeStepRef.current?.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
  }, [calcState?.pc]);

  if (!calcState) {
    return <div className="calculator"><div className="display">Loading...</div></div>;
  }

  const annunciatorNames = ['user', 'shift', 'prgm', 'alpha', 'rad', 'grad'] as const;
  // SHIFT is a frontend-derived annunciator; the rest come from the backend CalcStateView.
  const annunciators: Record<typeof annunciatorNames[number], boolean> = {
    user:  calcState.annunciators.user,
    shift: shiftActive,
    prgm:  calcState.annunciators.prgm,
    alpha: calcState.annunciators.alpha,
    rad:   calcState.annunciators.rad,
    grad:  calcState.annunciators.grad,
  };
  const stackRows: [string, string][] = [
    ['X', calcState.x_str],
    ['Y', calcState.y_str],
    ['Z', calcState.z_str],
    ['T', calcState.t_str],
    ['L', calcState.lastx_str],
  ];

  // Phase 26 D-26.3 — modal preview replaces LCD content during accumulation.
  // Phase 26 Plan 04 CR-04a — when no modal is open, prefer
  // `calcState.display_override` (set by Phase 21 ROM ops AView/Prompt/View
  // and cleared by the next op) over `calcState.display_str`. Without this
  // the backend's display_override projection is wired through IPC but the
  // React render path drops it — AVIEW / PROMPT / VIEW produce no visible
  // effect. Precedence order: modal preview > display_override > display_str.
  const displayText: string = pendingInput
    ? renderModalLcd(pendingInput)
    : (calcState.display_override ?? calcState.display_str);

  return (
    <div
      className={`calculator${isIos ? ' calculator-safe-area' : ''}`}
      data-isios={isIos || undefined}
    >
      {/* Phase 48 D-48.1 — title bar with gear icon and ? help button.
          The gear icon uses onMouseDown + e.stopPropagation() to prevent the
          SettingsPanel's click-outside mousedown listener from immediately
          re-closing the panel when the gear icon is clicked (RESEARCH.md pitfall). */}
      <div className="calculator-title-bar">
        <span className="calculator-title-bar-spacer" />
        <button
          className="help-icon-btn"
          aria-label="Open function reference"
          onClick={() => { setHelpOpen(true); setSettingsOpen(false); setOnboardingOpen(false); }}
        >
          ?
        </button>
        <button
          className="settings-gear-btn"
          aria-label="Open settings"
          aria-expanded={settingsOpen}
          onMouseDown={(e) => { e.stopPropagation(); setSettingsOpen(prev => !prev); if (!settingsOpen) setHelpOpen(false); }}
        >
          &#9881;
        </button>
        <SettingsPanel
          open={settingsOpen}
          onClose={() => setSettingsOpen(false)}
          currentTheme={theme}
          onThemeChange={handleThemeChange}
          onShowOnboarding={handleShowOnboarding}
          isMacos={isMacos}
          currentLaunchMode={macosLaunchMode}
          onLaunchModeChange={handleLaunchModeChange}
        />
      </div>
      <div className="annunciators">
        {annunciatorNames.map(name => (
          <span
            key={name}
            className={`annunciator annunciator-${name}${annunciators[name] ? ' active' : ''}`}
          >
            {name.toUpperCase()}
          </span>
        ))}
      </div>
      <div className="display" data-displaytext={displayText}><Display14Seg text={displayText} /></div>
      {toast && (
        <div key={toast.seq} className="toast" role="status">{toast.msg}</div>
      )}
      {errorMessage && (
        <div className="error-row" role="alert">{errorMessage}</div>
      )}
      {/* Phase 55 Plan 05 — Collapsible stack panel on iOS (TOUCH-10).
          On iOS: X row always visible; Y/Z/T/L wrapped in .stack-panel-collapsible
          controlled by stackExpanded. Chevron toggle is 44pt (TOUCH-10 contract).
          On desktop: full stack always expanded, no chevron. */}
      <div className="stack-panel">
        {isIos ? (
          <>
            {/* X row always visible on iOS */}
            <div className="stack-row">
              <span className="stack-label">X:</span>
              <span>{calcState.x_str}</span>
            </div>
            {/* Y/Z/T/L collapsible wrapper */}
            <div className={`stack-panel-collapsible${stackExpanded ? '' : ' collapsed'}`}>
              {stackRows.slice(1).map(([label, value]) => (
                <div key={label} className="stack-row">
                  <span className="stack-label">{label}:</span>
                  <span>{value}</span>
                </div>
              ))}
            </div>
            {/* 44pt chevron toggle */}
            <div className="stack-panel-toggle">
              <span>{stackExpanded ? 'Stack' : 'Stack (collapsed)'}</span>
              <button
                className="stack-panel-toggle-btn"
                aria-label={stackExpanded ? 'Collapse stack' : 'Expand stack'}
                onClick={() => setStackExpanded(e => !e)}
              >
                {stackExpanded ? '▲' : '▼'}
              </button>
            </div>
          </>
        ) : (
          stackRows.map(([label, value]) => (
            <div key={label} className="stack-row">
              <span className="stack-label">{label}:</span>
              <span>{value}</span>
            </div>
          ))
        )}
      </div>
      <Keyboard
        onKey={handleClick}
        busyRef={busyRef}
        shiftActive={shiftActive}
        alphaActive={calcState.annunciators.alpha}
        userActive={calcState.annunciators.user}
        userKeymap={calcState.user_keymap}
        gradientColors={THEME_GRADIENTS[theme] || THEME_GRADIENTS['dark']}
        isIos={isIos}
        onPointerDown={(key) => {
          // Phase 55 Plan 03: per-key haptics + audio resume (TOUCH-05, TOUCH-06, TOUCH-08).
          // Both calls are iOS-gated and silently catch on desktop.
          // Audio resume MUST be called first — must be inside a user-gesture handler.
          if (isIos) {
            // Lazily create the AudioContext on the first touch gesture so that the
            // constructor itself is also inside a user-gesture context (some browsers
            // require this). The context is held in audioCtxRef for subsequent calls.
            if (!audioCtxRef.current) {
              audioCtxRef.current = new AudioContext();
            }
            void ensureAudioResumed(audioCtxRef.current, audioResumedRef);
            void triggerHaptic(key, true);
          }
        }}
      />
      {/* Phase 55 Plan 04 — AlphaTouchInput: iOS-gated touch text entry bar (TOUCH-04 + D-55.2).
          Renders when isIos AND (ALPHA-register mode OR backend modal-label prompt is active).
          Routes via the EXISTING alpha_<X> dispatch and submit_modal_with_label IPC paths.
          Desktop (isIos=false) renders nothing — existing physical-keyboard path unchanged.
          XEQ/GTO/LBL/CLP/ASN-label modals (xeq_name, clp, assign_label) do NOT show this bar
          — per user decision (TOUCH-04): name entry uses the on-screen HP-41 keypad directly,
          with ENTER typing 'N' and ALPHA terminating (see handleClick modal-routing above). */}
      {isIos && (calcState.annunciators.alpha || calcState.modal_requires_alpha_label) && (
        <AlphaTouchInput
          isAlphaMode={calcState.annunciators.alpha}
          isModalLabelMode={calcState.modal_requires_alpha_label}
          modalPrompt={calcState.modal_prompt}
          onDispatch={dispatchKeyId}
          onSubmitLabel={(label) => invoke('submit_modal_with_label', { label })}
        />
      )}
      {/* Phase 55 Plan 05 — PRGM panel: bottom sheet on iOS, inline panel on desktop.
          iOS: pull-up sheet with active-step highlight + scroll-into-view.
          Desktop: existing inline .prgm-panel (byte-for-byte unchanged). */}
      {isIos ? (
        <BottomSheet
          id="prgm-sheet"
          title="PROGRAM"
          visible={calcState.annunciators.prgm}
          emptyText="Program memory empty."
        >
          {calcState.program_steps.map((step, i) => (
            <div
              key={i}
              ref={calcState.pc === i ? activeStepRef : null}
              className={`step-row${calcState.pc === i ? ' step-active' : ''}`}
            >
              {step}
            </div>
          ))}
        </BottomSheet>
      ) : (
        calcState.annunciators.prgm && (
          <div className="prgm-panel">
            <div className="prgm-panel-header">
              PRGM &#8212; {calcState.program_steps.length - 1}{' '}
              {calcState.program_steps.length - 1 === 1 ? 'step' : 'steps'}
            </div>
            <div className="prgm-panel-content">
              {calcState.program_steps.map((step, i) => (
                <div
                  key={i}
                  ref={calcState.pc === i ? activeStepRef : null}
                  className={`step-row${calcState.pc === i ? ' step-active' : ''}`}
                >
                  {step}
                </div>
              ))}
            </div>
          </div>
        )
      )}
      {/* Phase 55 Plan 05 — Print panel: bottom sheet on iOS, inline panel on desktop.
          iOS: pull-up sheet visible when printLog.length > 0.
          Desktop: existing .print-panel gated by printPanelOpen (byte-for-byte unchanged). */}
      {isIos ? (
        <BottomSheet
          id="print-sheet"
          title="PRINT LOG"
          visible={printLog.length > 0}
          emptyText="No print output yet."
        >
          {printLog.map((line, i) => (
            <div key={i} className="print-line">{line}</div>
          ))}
          <div ref={printEndRef} />
        </BottomSheet>
      ) : (
        printPanelOpen && (
          <div className="print-panel">
            <div className="print-panel-header">
              <span>PRINT</span>
              <button className="print-panel-close" onClick={() => setPrintPanelOpen(false)}>×</button>
            </div>
            <div className="print-panel-content">
              {printLog.map((line, i) => (
                <div key={i} className="print-line">{line}</div>
              ))}
              <div ref={printEndRef} />
            </div>
          </div>
        )
      )}
      {/* Phase 26 D-26.8 — `?` help overlay. The component returns null when
          open=false, so unconditional placement in the tree is safe. Anchored
          inside `.calculator` (position: relative) so the overlay's `position:
          absolute` covers the calculator footprint only, not the page. */}
      <HelpOverlay open={helpOpen} onClose={() => setHelpOpen(false)} />
      {/* Phase 50 D-50.4/D-50.6 — multi-program picker overlay (mutually exclusive
          with help and wizard overlays, same z-index: 60). Renders when pickerData
          is non-null (set by importRawDialog on multi-program .raw archive response). */}
      {pickerData && (
        <RawPickerOverlay
          programs={pickerData.programs}
          onConfirm={handlePickerConfirm}
          onClose={handlePickerClose}
        />
      )}
      {/* Phase 49 ONBOARD-01 — first-run onboarding wizard overlay.
          Mutual exclusion with help/settings enforced via onboardingOpen state.
          isFirstRun=true blocks Esc dismiss (D-49.9); re-open mode allows it.
          Full 5-panel implementation provided by Plan 49-03; this plan wires
          the state management and renders the component. */}
      <OnboardingWizard
        open={onboardingOpen}
        onClose={handleOnboardingClose}
        isFirstRun={isFirstRun}
      />
    </div>
  );
}

export default App;
