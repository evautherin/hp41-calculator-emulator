// Phase 55 Plan 03 — Haptics helper module (TOUCH-05, TOUCH-06, TOUCH-08)
//
// CRITICAL: The @tauri-apps/plugin-haptics 2.3.2 API uses BARE LOWERCASE STRINGS.
// The UI-SPEC pseudo-code showed the WRONG object form { style: 'Light' }.
// CORRECT: impactFeedback('light' | 'medium' | 'heavy')
// CORRECT: notificationFeedback('error')
// See 55-RESEARCH.md Pattern 3 for the verified API signature.
//
// All haptic calls are iOS-gated by the caller (isIos flag from Plan 01).
// On desktop, impactFeedback / notificationFeedback IPC will error (plugin not
// registered without #[cfg(mobile)]) — catch guards silence the error silently.

import { impactFeedback, notificationFeedback } from '@tauri-apps/plugin-haptics';
import type { KeyDef } from './Keyboard';

// Medium-tier key ids: function-group entry points that benefit from stronger
// feedback. Variant 'enter' is handled separately by the variant check.
const MEDIUM_KEY_IDS = new Set([
  'sto_prompt',
  'rcl_prompt',
  'xeq_prompt',
  'gto_prompt',
  'r_s',
  'rtn',
  'sst',
  'bst',
]);

/**
 * Pure classifier — returns the haptic tier for a given key.
 * No side effects; trivially unit-testable.
 *
 * Tier rules (from 55-UI-SPEC.md Haptics Contract + 55-RESEARCH.md Pattern 3):
 *   variant === 'shift'                                      → 'heavy'
 *   variant === 'enter' OR id ∈ MEDIUM_KEY_IDS              → 'medium'
 *   otherwise (incl. variant 'top', digits, math functions) → 'light'
 */
export function getHapticTier(key: KeyDef): 'light' | 'medium' | 'heavy' {
  if (key.variant === 'shift') {
    return 'heavy';
  }
  if (key.variant === 'enter' || MEDIUM_KEY_IDS.has(key.id)) {
    return 'medium';
  }
  return 'light';
}

/**
 * Fire a tier-appropriate impact haptic on iOS.
 * No-op (and silently catch) on desktop — the plugin is not registered there.
 *
 * @param key    - The key that was pressed (used to derive the tier)
 * @param isIos  - True only when running on iOS (from Plan 01 isIos state)
 */
export async function triggerHaptic(key: KeyDef, isIos: boolean): Promise<void> {
  if (!isIos) return;
  const tier = getHapticTier(key);
  await impactFeedback(tier).catch(() => {
    // Silently ignore — plugin not available on desktop, or unsupported device
  });
}

/**
 * Guarded error haptic — fires notificationFeedback('error') once when the
 * display transitions into a DATA ERROR or NO ROOM state. Resets the guard
 * when the display returns to a non-error value.
 *
 * The caller holds the `firedRef` (a React useRef<boolean>) so the guard
 * persists across re-renders without causing state updates.
 *
 * Pitfall 7 (T-55-06): without this guard the haptic would re-fire on every
 * clock tick or re-render while the error is displayed.
 *
 * @param displayStr - The current display_str from CalcStateView
 * @param isIos      - iOS gate (mirrors triggerHaptic)
 * @param firedRef   - Mutable ref object: { current: boolean }
 */
export async function maybeFireErrorHaptic(
  displayStr: string,
  isIos: boolean,
  firedRef: { current: boolean },
): Promise<void> {
  const isError =
    displayStr.includes('DATA ERROR') || displayStr.includes('NO ROOM');

  if (isError) {
    if (isIos && !firedRef.current) {
      firedRef.current = true;
      await notificationFeedback('error').catch(() => {
        // Silently ignore — plugin not available on desktop
      });
    }
  } else {
    // Display cleared — reset the guard for the next error transition
    firedRef.current = false;
  }
}

/**
 * One-shot audio resume helper. Resumes a suspended AudioContext inside a
 * user-gesture handler (required by WKWebView / iOS policy). After the first
 * successful resume, the `audioResumedRef` guard prevents redundant calls.
 *
 * MUST be called inside a pointerdown handler to satisfy iOS gesture
 * requirements. Calling it outside a gesture handler will silently fail.
 *
 * @param audioCtx       - The AudioContext to resume
 * @param audioResumedRef - One-shot guard: { current: boolean }
 */
export async function ensureAudioResumed(
  audioCtx: AudioContext,
  audioResumedRef: { current: boolean },
): Promise<void> {
  if (audioResumedRef.current) return;
  if (audioCtx.state === 'suspended') {
    await audioCtx.resume().catch(() => {
      // Silently ignore — resume() can reject outside a valid gesture context
      // or when the iOS audio session is interrupted. Mirrors the internal
      // .catch() guards in triggerHaptic / maybeFireErrorHaptic so a rejected
      // resume() cannot surface as an unhandled promise rejection on iOS.
    });
  }
  audioResumedRef.current = true;
}
