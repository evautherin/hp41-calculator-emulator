// Phase 55 Plan 04 — AlphaTouchInput component (TOUCH-04 + D-55.2).
//
// Renders a fixed bar at the bottom of the screen for text entry on iOS:
//   - When isAlphaMode=true: shows "ALPHA REGISTER" header; each typed char
//     [A-Z0-9 ] dispatches alpha_<X> via onDispatch; Backspace → clx;
//     Done dispatches alpha_toggle to exit ALPHA mode.
//   - When isModalLabelMode=true: shows modalPrompt as header; accumulates
//     typed characters in local state; Done calls onSubmitLabel(accumulated).
//   - When isFrontendModalMode=true (Phase 55 Plan 06 gap-fix, TOUCH-04):
//     shows frontendModalPrompt as header; each typed char calls
//     onFrontendModalChar(uppercased) so App.tsx can route through
//     handleModalKey/applyModalResult (pendingInput.acc accumulates there);
//     Backspace calls onFrontendModalBackspace; Done calls onFrontendModalDone.
//     This mode covers XEQ/GTO/LBL/CLP/ASN frontend-modal kinds
//     (xeq_name, clp, assign_label) that set pendingInput but do NOT set
//     calcState.modal_requires_alpha_label — they were previously invisible
//     on iOS because the render gate only checked the backend flag.
//
// The bar tracks the iOS software keyboard via visualViewport so it stays
// above the keyboard at all times (Pattern 5 from RESEARCH, Pitfall 4 fix).
//
// Security: all characters are uppercased and filtered to [A-Z0-9 ] before
// forming alpha_<X> keyIds — identical to the desktop physical keyboard path
// in App.tsx resolveKeyId (T-55-07 mitigation). Modal label submitted verbatim
// to the existing submit_modal_with_label command which validates length (T-55-08).

import { useState, useEffect, useRef } from 'react';

export interface AlphaTouchInputProps {
  isAlphaMode: boolean;           // calcState.annunciators.alpha
  isModalLabelMode: boolean;      // calcState.modal_requires_alpha_label
  modalPrompt: string | null;     // calcState.modal_prompt
  onDispatch: (keyId: string) => void;     // App.tsx dispatchKeyId
  onSubmitLabel: (label: string) => void;  // invoke('submit_modal_with_label')
  // Phase 55 Plan 06 gap-fix: frontend modal mode (xeq_name / clp / assign_label)
  isFrontendModalMode: boolean;            // pendingInput.kind ∈ {xeq_name, clp, assign_label}
  frontendModalPrompt: string | null;      // derived from pendingInput.kind + dispatchPrefix
  onFrontendModalChar: (ch: string) => void;   // routes char through handleModalKey/applyModalResult
  onFrontendModalBackspace: () => void;         // routes Backspace through handleModalKey/applyModalResult
  onFrontendModalDone: () => void;              // routes Enter through handleModalKey/applyModalResult
}

// SettingsPanel analog: early-return null when no mode is active.
export default function AlphaTouchInput({
  isAlphaMode,
  isModalLabelMode,
  modalPrompt,
  onDispatch,
  onSubmitLabel,
  isFrontendModalMode,
  frontendModalPrompt,
  onFrontendModalChar,
  onFrontendModalBackspace,
  onFrontendModalDone,
}: AlphaTouchInputProps) {
  // Guard: do not render when no mode is active.
  if (!isAlphaMode && !isModalLabelMode && !isFrontendModalMode) return null;

  return (
    <AlphaTouchInputInner
      isAlphaMode={isAlphaMode}
      isModalLabelMode={isModalLabelMode}
      modalPrompt={modalPrompt}
      onDispatch={onDispatch}
      onSubmitLabel={onSubmitLabel}
      isFrontendModalMode={isFrontendModalMode}
      frontendModalPrompt={frontendModalPrompt}
      onFrontendModalChar={onFrontendModalChar}
      onFrontendModalBackspace={onFrontendModalBackspace}
      onFrontendModalDone={onFrontendModalDone}
    />
  );
}

// Inner component mounts only when at least one mode is active.
// Separated to allow hooks to run unconditionally (Rules of Hooks compliance).
function AlphaTouchInputInner({
  isAlphaMode,
  isModalLabelMode,
  modalPrompt,
  onDispatch,
  onSubmitLabel,
  isFrontendModalMode,
  frontendModalPrompt,
  onFrontendModalChar,
  onFrontendModalBackspace,
  onFrontendModalDone,
}: AlphaTouchInputProps) {
  // Local input value state — controlled input.
  // In ALPHA mode: cleared after each char dispatch.
  // In modal-label mode: accumulated until Done is pressed.
  // In frontend-modal mode: cleared after each char (state lives in App.tsx pendingInput.acc).
  const [inputValue, setInputValue] = useState('');
  const [bottomOffset, setBottomOffset] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);

  // visualViewport keyboard-tracking (RESEARCH Pattern 5 / Pitfall 4).
  // Computes keyboardHeight = window.innerHeight - vv.height - vv.offsetTop
  // (NOT vv.height alone — Tauri #10631 workaround).
  useEffect(() => {
    const vv = window.visualViewport;
    if (!vv) return;
    const updatePosition = () => {
      const keyboardHeight = window.innerHeight - vv.height - vv.offsetTop;
      setBottomOffset(Math.max(keyboardHeight, 0));
    };
    vv.addEventListener('resize', updatePosition);
    vv.addEventListener('scroll', updatePosition);
    // Initial measurement
    updatePosition();
    return () => {
      vv.removeEventListener('resize', updatePosition);
      vv.removeEventListener('scroll', updatePosition);
    };
  }, []);

  // Auto-focus the input when the bar mounts (keyboard pops up on iOS).
  useEffect(() => {
    inputRef.current?.focus();
  }, []);

  // Reset local input value when mode changes (e.g. modal closes).
  useEffect(() => {
    setInputValue('');
  }, [isAlphaMode, isModalLabelMode, isFrontendModalMode]);

  // Handle input change event.
  // In ALPHA mode: dispatch one alpha_<X> per new character added, then clear.
  // In modal-label mode: update accumulated local state.
  // In frontend-modal mode: call onFrontendModalChar per new character, then clear.
  const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const newValue = e.target.value;
    if (isAlphaMode) {
      // Dispatch each new character in [A-Z0-9 ] (uppercased, T-55-07 filter).
      const diff = newValue.slice(inputValue.length);
      for (const rawCh of diff) {
        const ch = rawCh.toUpperCase();
        if (/^[A-Z0-9 ]$/.test(ch)) {
          onDispatch(`alpha_${ch}`);
        }
      }
      // Clear the input after dispatching so successive chars are always "diff".
      setInputValue('');
    } else if (isFrontendModalMode) {
      // Frontend-modal mode: route each new char to App.tsx via onFrontendModalChar.
      // State (pending.acc) lives in App.tsx; we just clear and pass through.
      const diff = newValue.slice(inputValue.length);
      for (const rawCh of diff) {
        const ch = rawCh.toUpperCase();
        // Pass all printable chars (handleModalKey's isPrintableChar in pending_input.ts
        // handles the actual validation); uppercase to match on-screen keypad convention.
        if (ch.length === 1) {
          onFrontendModalChar(ch);
        }
      }
      // Clear so successive chars always produce a non-empty diff.
      setInputValue('');
    } else {
      // modal-label mode: accumulate
      setInputValue(newValue);
    }
  };

  // Handle keydown — intercept Backspace before the browser deletes characters.
  const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === 'Backspace') {
      if (isAlphaMode) {
        // ALPHA mode: Backspace → clx (clear X register, removes last alpha char in HP-41).
        e.preventDefault();
        onDispatch('clx');
      } else if (isFrontendModalMode) {
        // Frontend-modal mode: route Backspace through App.tsx → handleModalKey → applyModalResult.
        e.preventDefault();
        onFrontendModalBackspace();
      } else {
        // modal-label mode: remove last char from accumulated local state.
        e.preventDefault();
        setInputValue(prev => prev.slice(0, -1));
      }
    }
  };

  // Handle Done button.
  const handleDone = () => {
    if (isAlphaMode) {
      // Exit ALPHA mode.
      onDispatch('alpha_toggle');
    } else if (isFrontendModalMode) {
      // Frontend-modal mode: submit via App.tsx → handleModalKey('Enter') → applyModalResult.
      onFrontendModalDone();
    } else {
      // Submit the accumulated label to the existing submit_modal_with_label command.
      onSubmitLabel(inputValue);
    }
  };

  // Header label: "ALPHA REGISTER" in ALPHA mode; frontendModalPrompt in frontend-modal mode;
  // modalPrompt in modal-label mode.
  const headerLabel = isAlphaMode
    ? 'ALPHA REGISTER'
    : isFrontendModalMode
      ? (frontendModalPrompt ?? 'ENTER LABEL')
      : (modalPrompt ?? 'ENTER LABEL');

  return (
    <div
      className="alpha-touch-input-bar"
      style={{ bottom: `${bottomOffset}px` }}
    >
      <span className="alpha-touch-input-label">{headerLabel}</span>
      <input
        ref={inputRef}
        type="text"
        value={inputValue}
        onChange={handleChange}
        onKeyDown={handleKeyDown}
        inputMode="text"
        autoComplete="off"
        autoCorrect="off"
        spellCheck={false}
        autoFocus
        aria-label={headerLabel}
      />
      <button type="button" onClick={handleDone}>
        Done
      </button>
    </div>
  );
}
