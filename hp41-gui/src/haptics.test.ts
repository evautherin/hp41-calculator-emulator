// Phase 55 Plan 03 — Unit tests for haptics.ts
// TDD RED phase: covers the 4 behaviors described in the plan.
//
// 1. getHapticTier(key with variant 'shift') === 'heavy'
// 2. getHapticTier(key with variant 'enter' or id in medium-set) === 'medium'
// 3. getHapticTier(digit key, variant undefined/'top') === 'light'
// 4. error-haptic guard fires notificationFeedback('error') only on transition
//    into an error display and resets when the error clears (no double-fire)

import { describe, it, expect, vi, beforeEach } from 'vitest';

// --- Plugin mock -----------------------------------------------------------
// Mock '@tauri-apps/plugin-haptics' before importing haptics.ts so the
// module sees the mocked versions. Both functions return a resolved Promise.

const mockImpactFeedback = vi.fn().mockResolvedValue(undefined);
const mockNotificationFeedback = vi.fn().mockResolvedValue(undefined);

vi.mock('@tauri-apps/plugin-haptics', () => ({
  impactFeedback: (style: string) => mockImpactFeedback(style),
  notificationFeedback: (type: string) => mockNotificationFeedback(type),
}));

// Import AFTER mock is established
import { getHapticTier, triggerHaptic, maybeFireErrorHaptic } from './haptics';
import type { KeyDef } from './Keyboard';

// --- Helpers ---------------------------------------------------------------

function makeKey(overrides: Partial<KeyDef>): KeyDef {
  return {
    id: 'test_key',
    label: 'TEST',
    row: 1,
    col: 0,
    ...overrides,
  };
}

// --- Tests -----------------------------------------------------------------

describe('getHapticTier', () => {
  it('returns "heavy" for variant shift', () => {
    const key = makeKey({ id: 'shift', variant: 'shift' });
    expect(getHapticTier(key)).toBe('heavy');
  });

  it('returns "medium" for variant enter', () => {
    const key = makeKey({ id: 'enter', variant: 'enter' });
    expect(getHapticTier(key)).toBe('medium');
  });

  it('returns "medium" for sto_prompt id', () => {
    expect(getHapticTier(makeKey({ id: 'sto_prompt' }))).toBe('medium');
  });

  it('returns "medium" for rcl_prompt id', () => {
    expect(getHapticTier(makeKey({ id: 'rcl_prompt' }))).toBe('medium');
  });

  it('returns "medium" for xeq_prompt id', () => {
    expect(getHapticTier(makeKey({ id: 'xeq_prompt' }))).toBe('medium');
  });

  it('returns "medium" for gto_prompt id', () => {
    expect(getHapticTier(makeKey({ id: 'gto_prompt' }))).toBe('medium');
  });

  it('returns "medium" for r_s id', () => {
    expect(getHapticTier(makeKey({ id: 'r_s' }))).toBe('medium');
  });

  it('returns "medium" for rtn id', () => {
    expect(getHapticTier(makeKey({ id: 'rtn' }))).toBe('medium');
  });

  it('returns "medium" for sst id', () => {
    expect(getHapticTier(makeKey({ id: 'sst' }))).toBe('medium');
  });

  it('returns "medium" for bst id', () => {
    expect(getHapticTier(makeKey({ id: 'bst' }))).toBe('medium');
  });

  it('returns "light" for digit key (variant top)', () => {
    const key = makeKey({ id: 'user_mode', variant: 'top' });
    expect(getHapticTier(key)).toBe('light');
  });

  it('returns "light" for a digit key (no variant)', () => {
    const key = makeKey({ id: '5', row: 6, col: 2 });
    expect(getHapticTier(key)).toBe('light');
  });

  it('returns "light" for math key (sin)', () => {
    const key = makeKey({ id: 'sin', row: 2, col: 2 });
    expect(getHapticTier(key)).toBe('light');
  });
});

describe('triggerHaptic', () => {
  beforeEach(() => {
    mockImpactFeedback.mockClear();
  });

  it('calls impactFeedback with correct tier when isIos=true', async () => {
    const key = makeKey({ id: 'shift', variant: 'shift' });
    await triggerHaptic(key, true);
    expect(mockImpactFeedback).toHaveBeenCalledWith('heavy');
  });

  it('does NOT call impactFeedback when isIos=false', async () => {
    const key = makeKey({ id: 'shift', variant: 'shift' });
    await triggerHaptic(key, false);
    expect(mockImpactFeedback).not.toHaveBeenCalled();
  });

  it('uses correct tier for enter key', async () => {
    const key = makeKey({ id: 'enter', variant: 'enter' });
    await triggerHaptic(key, true);
    expect(mockImpactFeedback).toHaveBeenCalledWith('medium');
  });

  it('uses correct tier for digit key', async () => {
    const key = makeKey({ id: '7', row: 5, col: 1 });
    await triggerHaptic(key, true);
    expect(mockImpactFeedback).toHaveBeenCalledWith('light');
  });
});

describe('maybeFireErrorHaptic — error-haptic double-fire guard', () => {
  beforeEach(() => {
    mockNotificationFeedback.mockClear();
  });

  it('fires notificationFeedback("error") on first DATA ERROR display', async () => {
    const firedRef = { current: false };
    await maybeFireErrorHaptic('DATA ERROR', true, firedRef);
    expect(mockNotificationFeedback).toHaveBeenCalledWith('error');
    expect(firedRef.current).toBe(true);
  });

  it('does NOT fire again on second DATA ERROR display (guard active)', async () => {
    const firedRef = { current: true }; // already fired
    await maybeFireErrorHaptic('DATA ERROR', true, firedRef);
    expect(mockNotificationFeedback).not.toHaveBeenCalled();
  });

  it('fires notificationFeedback("error") on NO ROOM display', async () => {
    const firedRef = { current: false };
    await maybeFireErrorHaptic('NO ROOM', true, firedRef);
    expect(mockNotificationFeedback).toHaveBeenCalledWith('error');
    expect(firedRef.current).toBe(true);
  });

  it('resets the guard when display clears (non-error string)', async () => {
    const firedRef = { current: true };
    await maybeFireErrorHaptic('0.0000', true, firedRef);
    expect(firedRef.current).toBe(false);
    expect(mockNotificationFeedback).not.toHaveBeenCalled();
  });

  it('does NOT fire when isIos=false even for DATA ERROR', async () => {
    const firedRef = { current: false };
    await maybeFireErrorHaptic('DATA ERROR', false, firedRef);
    expect(mockNotificationFeedback).not.toHaveBeenCalled();
    // Guard should still be updated for consistency (ref tracks state regardless)
  });

  it('fires once on transition, not on subsequent re-renders with same error', async () => {
    const firedRef = { current: false };
    // First render with error — fires
    await maybeFireErrorHaptic('DATA ERROR', true, firedRef);
    expect(mockNotificationFeedback).toHaveBeenCalledTimes(1);
    // Second render with same error — does NOT fire again
    await maybeFireErrorHaptic('DATA ERROR', true, firedRef);
    expect(mockNotificationFeedback).toHaveBeenCalledTimes(1);
    // After clearing, ref resets
    await maybeFireErrorHaptic('0.0000', true, firedRef);
    expect(firedRef.current).toBe(false);
    // Next error fires again
    await maybeFireErrorHaptic('DATA ERROR', true, firedRef);
    expect(mockNotificationFeedback).toHaveBeenCalledTimes(2);
  });
});
