// Tests for the global-shortcut recorder: the pure accelerator helpers
// (accelFromEvent / formatAccel) and the capture-overlay component behavior.
//
// Vitest runs with globals:false, so imports are explicit and each test cleans up
// (the component portals into document.body — afterEach(cleanup) prevents leakage).

import { describe, it, expect, afterEach, vi } from 'vitest';
import { render, fireEvent, cleanup } from '@testing-library/react';
import { ShortcutRecorder, accelFromEvent, formatAccel, type CapturedKey } from './ShortcutRecorder';

afterEach(cleanup);

const ev = (over: Partial<CapturedKey>): CapturedKey => ({
  ctrlKey: false,
  altKey: false,
  shiftKey: false,
  metaKey: false,
  key: '',
  code: '',
  ...over,
});

describe('accelFromEvent', () => {
  it('builds the full ⌃⌥⌘ combo in canonical modifier order', () => {
    const accel = accelFromEvent(
      ev({ ctrlKey: true, altKey: true, metaKey: true, code: 'KeyH', key: 'h' }),
    );
    expect(accel).toBe('Control+Alt+Command+H');
  });

  it('handles a single modifier + letter', () => {
    expect(accelFromEvent(ev({ shiftKey: true, code: 'KeyA', key: 'a' }))).toBe('Shift+A');
  });

  it('maps digits and function keys', () => {
    expect(accelFromEvent(ev({ ctrlKey: true, code: 'Digit5', key: '5' }))).toBe('Control+5');
    expect(accelFromEvent(ev({ ctrlKey: true, code: 'F7', key: 'F7' }))).toBe('Control+F7');
    expect(accelFromEvent(ev({ altKey: true, code: 'Space', key: ' ' }))).toBe('Alt+Space');
  });

  it('rejects a bare key with no modifier', () => {
    expect(accelFromEvent(ev({ code: 'KeyH', key: 'h' }))).toBeNull();
  });

  it('rejects a bare modifier press', () => {
    expect(accelFromEvent(ev({ ctrlKey: true, code: 'ControlLeft', key: 'Control' }))).toBeNull();
  });
});

describe('formatAccel', () => {
  it('renders accelerator tokens as macOS glyphs', () => {
    expect(formatAccel('Control+Alt+Command+H')).toBe('⌃⌥⌘H');
    expect(formatAccel('Shift+5')).toBe('⇧5');
    expect(formatAccel('Control+F7')).toBe('⌃F7');
  });

  it('returns empty string for empty input', () => {
    expect(formatAccel('')).toBe('');
  });
});

describe('ShortcutRecorder component', () => {
  it('shows the current shortcut until a new one is captured', () => {
    const { getByText } = render(
      <ShortcutRecorder current="Control+Alt+Command+H" onConfirm={vi.fn()} onCancel={vi.fn()} />,
    );
    expect(getByText('⌃⌥⌘H')).toBeTruthy();
  });

  it('captures a combo, enables Save, and confirms with the accelerator', () => {
    const onConfirm = vi.fn();
    const { getByText } = render(
      <ShortcutRecorder current="Control+Alt+Command+H" onConfirm={onConfirm} onCancel={vi.fn()} />,
    );
    fireEvent.keyDown(window, { ctrlKey: true, shiftKey: true, code: 'KeyK', key: 'k' });
    // Preview updates to the captured combo.
    expect(getByText('⌃⇧K')).toBeTruthy();
    fireEvent.click(getByText('Save'));
    expect(onConfirm).toHaveBeenCalledWith('Control+Shift+K');
  });

  it('keeps Save disabled until a valid combo is captured', () => {
    const onConfirm = vi.fn();
    const { getByText } = render(
      <ShortcutRecorder current="Control+Alt+Command+H" onConfirm={onConfirm} onCancel={vi.fn()} />,
    );
    fireEvent.click(getByText('Save'));
    expect(onConfirm).not.toHaveBeenCalled();
  });

  it('cancels on Escape and on the Cancel button', () => {
    const onCancel = vi.fn();
    const { getByText } = render(
      <ShortcutRecorder current="Control+Alt+Command+H" onConfirm={vi.fn()} onCancel={onCancel} />,
    );
    fireEvent.keyDown(window, { key: 'Escape', code: 'Escape' });
    expect(onCancel).toHaveBeenCalledTimes(1);
    fireEvent.click(getByText('Cancel'));
    expect(onCancel).toHaveBeenCalledTimes(2);
  });
});
