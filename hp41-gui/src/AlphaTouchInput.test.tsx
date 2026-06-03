// Phase 55 Plan 04 — Unit tests for AlphaTouchInput.tsx
// TDD RED phase: covers the 5 behaviors described in the plan.
//
// 1. Renders the bar when isAlphaMode=true (header "ALPHA REGISTER").
// 2. Renders the bar when isModalLabelMode=true (header shows modalPrompt).
// 3. Renders nothing when both isAlphaMode and isModalLabelMode are false.
// 4. In ALPHA mode, typing "A" calls onDispatch('alpha_A'); Backspace calls onDispatch('clx').
// 5. In modal-label mode, pressing Done calls onSubmitLabel with the accumulated input value.
//
// Note: Phase 55 Plan 06 previously added tests 6-10 for an isFrontendModalMode prop
// (iOS software-keyboard bar for XEQ/GTO/LBL/CLP/ASN modals). That feature was rejected
// on-device (keyboard pushes layout, blind entry). The approach is superseded by keypad-only
// name entry (ENTER=N, ALPHA=terminate in App.tsx handleClick). Tests 6-10 are removed;
// the component is restored to its original two-mode form.

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
    alphaText?: string;
    onDispatch?: (keyId: string) => void;
    onSubmitLabel?: (label: string) => void;
  } = {},
) {
  const onDispatch = overrides.onDispatch ?? vi.fn();
  const onSubmitLabel = overrides.onSubmitLabel ?? vi.fn();
  return render(
    <AlphaTouchInput
      isAlphaMode={overrides.isAlphaMode ?? false}
      isModalLabelMode={overrides.isModalLabelMode ?? false}
      modalPrompt={overrides.modalPrompt ?? null}
      alphaText={overrides.alphaText ?? ''}
      onDispatch={onDispatch}
      onSubmitLabel={onSubmitLabel}
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

  // Test 3: renders nothing when both modes are false (component has no active mode)
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

});
