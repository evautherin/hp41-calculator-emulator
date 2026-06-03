// Phase 55 Plan 04 — Unit tests for AlphaTouchInput.tsx
// TDD RED phase: covers the 5 behaviors described in the plan.
//
// 1. Renders the bar when isAlphaMode=true (header "ALPHA REGISTER").
// 2. Renders the bar when isModalLabelMode=true (header shows modalPrompt).
// 3. Renders nothing when both isAlphaMode and isModalLabelMode are false.
// 4. In ALPHA mode, typing "A" calls onDispatch('alpha_A'); Backspace calls onDispatch('clx').
// 5. In modal-label mode, pressing Done calls onSubmitLabel with the accumulated input value.
//
// Phase 55 Plan 06 gap-fix — Frontend modal mode (TOUCH-04 / FRONTEND-MODAL):
// 6. Renders the bar when isFrontendModalMode=true with frontendModalPrompt as header.
// 7. Renders nothing when only isFrontendModalMode=false (and others also false).
// 8. In frontend-modal mode, typing "N" calls onFrontendModalChar('N').
// 9. In frontend-modal mode, Backspace calls onFrontendModalBackspace().
// 10. In frontend-modal mode, Done button calls onFrontendModalDone().

import { describe, it, expect, vi, afterEach } from 'vitest';
import { render, fireEvent, cleanup } from '@testing-library/react';
import AlphaTouchInput from './AlphaTouchInput';

// --- Plugin mocks (required even if AlphaTouchInput itself doesn't call them,
//     to prevent "module not found" errors in Vitest module resolution) --------

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn().mockResolvedValue(undefined),
}));

vi.mock('@tauri-apps/plugin-haptics', () => ({
  impactFeedback: vi.fn().mockResolvedValue(undefined),
  notificationFeedback: vi.fn().mockResolvedValue(undefined),
}));

// --- Cleanup ---------------------------------------------------------------

afterEach(cleanup);

// --- Helpers ---------------------------------------------------------------

function renderAlpha(
  overrides: {
    isAlphaMode?: boolean;
    isModalLabelMode?: boolean;
    modalPrompt?: string | null;
    onDispatch?: (keyId: string) => void;
    onSubmitLabel?: (label: string) => void;
    // Phase 55 Plan 06 gap-fix: frontend modal mode props
    isFrontendModalMode?: boolean;
    frontendModalPrompt?: string | null;
    onFrontendModalChar?: (ch: string) => void;
    onFrontendModalBackspace?: () => void;
    onFrontendModalDone?: () => void;
  } = {},
) {
  const onDispatch = overrides.onDispatch ?? vi.fn();
  const onSubmitLabel = overrides.onSubmitLabel ?? vi.fn();
  const onFrontendModalChar = overrides.onFrontendModalChar ?? vi.fn();
  const onFrontendModalBackspace = overrides.onFrontendModalBackspace ?? vi.fn();
  const onFrontendModalDone = overrides.onFrontendModalDone ?? vi.fn();
  return render(
    <AlphaTouchInput
      isAlphaMode={overrides.isAlphaMode ?? false}
      isModalLabelMode={overrides.isModalLabelMode ?? false}
      modalPrompt={overrides.modalPrompt ?? null}
      onDispatch={onDispatch}
      onSubmitLabel={onSubmitLabel}
      isFrontendModalMode={overrides.isFrontendModalMode ?? false}
      frontendModalPrompt={overrides.frontendModalPrompt ?? null}
      onFrontendModalChar={onFrontendModalChar}
      onFrontendModalBackspace={onFrontendModalBackspace}
      onFrontendModalDone={onFrontendModalDone}
    />,
  );
}

// --- Tests ------------------------------------------------------------------

describe('AlphaTouchInput', () => {
  // Test 1: renders bar with "ALPHA REGISTER" header in ALPHA mode
  it('renders the bar when isAlphaMode=true with "ALPHA REGISTER" header', () => {
    const { container, getByText } = renderAlpha({ isAlphaMode: true });
    expect(container.firstChild).not.toBeNull();
    expect(getByText('ALPHA REGISTER')).toBeTruthy();
  });

  // Test 2: renders bar with modalPrompt header in modal-label mode
  it('renders the bar when isModalLabelMode=true with modalPrompt as header', () => {
    const { container, getByText } = renderAlpha({
      isModalLabelMode: true,
      modalPrompt: 'FUNCTION NAME?',
    });
    expect(container.firstChild).not.toBeNull();
    expect(getByText('FUNCTION NAME?')).toBeTruthy();
  });

  // Test 3: renders nothing when both modes are false
  it('renders nothing when both isAlphaMode and isModalLabelMode are false', () => {
    const { container } = renderAlpha({ isAlphaMode: false, isModalLabelMode: false });
    expect(container.firstChild).toBeNull();
  });

  // Test 4a: typing "A" in ALPHA mode calls onDispatch('alpha_A')
  it('calls onDispatch("alpha_A") when typing A in ALPHA mode', () => {
    const onDispatch = vi.fn();
    const { getByRole } = renderAlpha({ isAlphaMode: true, onDispatch });
    const input = getByRole('textbox') as HTMLInputElement;
    // Simulate typing "A" — fire change event with target value "A"
    fireEvent.change(input, { target: { value: 'A' } });
    expect(onDispatch).toHaveBeenCalledWith('alpha_A');
  });

  // Test 4b: Backspace in ALPHA mode calls onDispatch('clx')
  it('calls onDispatch("clx") on Backspace key in ALPHA mode', () => {
    const onDispatch = vi.fn();
    const { getByRole } = renderAlpha({ isAlphaMode: true, onDispatch });
    const input = getByRole('textbox') as HTMLInputElement;
    fireEvent.keyDown(input, { key: 'Backspace' });
    expect(onDispatch).toHaveBeenCalledWith('clx');
  });

  // Test 5: Done button in modal-label mode calls onSubmitLabel with accumulated input
  it('calls onSubmitLabel with accumulated input on Done in modal-label mode', () => {
    const onSubmitLabel = vi.fn();
    const { getByRole, getByText } = renderAlpha({
      isModalLabelMode: true,
      modalPrompt: 'FUNCTION NAME?',
      onSubmitLabel,
    });
    const input = getByRole('textbox') as HTMLInputElement;
    // Type "MYPROG" into the input (use direct value assignment for accumulated)
    fireEvent.change(input, { target: { value: 'MYPROG' } });
    // Press Done
    const doneBtn = getByText('Done');
    fireEvent.click(doneBtn);
    expect(onSubmitLabel).toHaveBeenCalledWith('MYPROG');
  });

  // -----------------------------------------------------------------------
  // Phase 55 Plan 06 gap-fix — Frontend modal mode (TOUCH-04 fix)
  // Tests 6-10: isFrontendModalMode=true path for xeq_name/clp/assign_label
  // -----------------------------------------------------------------------

  // Test 6: renders bar with frontendModalPrompt header in frontend-modal mode
  it('renders the bar when isFrontendModalMode=true with frontendModalPrompt as header', () => {
    const { container, getByText } = renderAlpha({
      isFrontendModalMode: true,
      frontendModalPrompt: 'XEQ NAME?',
    });
    expect(container.firstChild).not.toBeNull();
    expect(getByText('XEQ NAME?')).toBeTruthy();
  });

  // Test 7: renders nothing when isFrontendModalMode=false (and others also false)
  it('renders nothing when isFrontendModalMode=false and other modes are also false', () => {
    const { container } = renderAlpha({
      isAlphaMode: false,
      isModalLabelMode: false,
      isFrontendModalMode: false,
    });
    expect(container.firstChild).toBeNull();
  });

  // Test 8: typing a character in frontend-modal mode calls onFrontendModalChar with uppercased char
  it('calls onFrontendModalChar with uppercase char when typing in frontend-modal mode', () => {
    const onFrontendModalChar = vi.fn();
    const { getByRole } = renderAlpha({
      isFrontendModalMode: true,
      frontendModalPrompt: 'XEQ NAME?',
      onFrontendModalChar,
    });
    const input = getByRole('textbox') as HTMLInputElement;
    // Simulate typing "n" (lowercase) — should call onFrontendModalChar('N')
    fireEvent.change(input, { target: { value: 'n' } });
    expect(onFrontendModalChar).toHaveBeenCalledWith('N');
  });

  // Test 9: Backspace in frontend-modal mode calls onFrontendModalBackspace
  it('calls onFrontendModalBackspace on Backspace key in frontend-modal mode', () => {
    const onFrontendModalBackspace = vi.fn();
    const { getByRole } = renderAlpha({
      isFrontendModalMode: true,
      frontendModalPrompt: 'XEQ NAME?',
      onFrontendModalBackspace,
    });
    const input = getByRole('textbox') as HTMLInputElement;
    fireEvent.keyDown(input, { key: 'Backspace' });
    expect(onFrontendModalBackspace).toHaveBeenCalledTimes(1);
  });

  // Test 10: Done button in frontend-modal mode calls onFrontendModalDone
  it('calls onFrontendModalDone when Done is pressed in frontend-modal mode', () => {
    const onFrontendModalDone = vi.fn();
    const { getByText } = renderAlpha({
      isFrontendModalMode: true,
      frontendModalPrompt: 'XEQ NAME?',
      onFrontendModalDone,
    });
    const doneBtn = getByText('Done');
    fireEvent.click(doneBtn);
    expect(onFrontendModalDone).toHaveBeenCalledTimes(1);
  });
});
