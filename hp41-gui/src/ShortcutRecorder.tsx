// Global-shortcut recorder overlay (macOS menu-bar hotkey configuration).
//
// Captures a key combination from a real keydown event and emits a Tauri
// accelerator string (e.g. "Control+Alt+Command+H") in the exact grammar the Rust
// `shortcut` module parses. The backend is the source of truth: App.tsx persists
// the captured accelerator via `set_pref`, and only updates the displayed value if
// the backend accepts AND registers it — so any token the Rust parser rejects never
// sticks.
//
// Rendered via createPortal to document.body so the iOS `transform: scale()` ancestor
// (ADR-v4.1-005) is not its containing block. The keydown listener runs in the
// capture phase and stops propagation, so the calculator's window keydown handler
// never fires while the recorder is open.

import { useEffect, useState } from 'react';
import { createPortal } from 'react-dom';

const MOD_SYMBOLS: Record<string, string> = {
    Control: '⌃',
    Alt: '⌥',
    Shift: '⇧',
    Command: '⌘',
};

/** Convert an accelerator string ("Control+Alt+Command+H") to macOS glyphs ("⌃⌥⌘H"). */
export function formatAccel(accel: string): string {
    if (!accel) return '';
    return accel
        .split('+')
        .map(part => MOD_SYMBOLS[part] ?? part)
        .join('');
}

/** The subset of a KeyboardEvent the pure capture logic needs (keeps it unit-testable). */
export type CapturedKey = {
    ctrlKey: boolean;
    altKey: boolean;
    shiftKey: boolean;
    metaKey: boolean;
    key: string;
    code: string;
};

/**
 * Map a physical key `code` (+ printable `key` fallback) to an accelerator key token
 * the Rust parser accepts, or `null` for a bare modifier / unsupported key.
 * Intentionally conservative — only letters, digits, F1–F12 and Space — so every
 * emitted token is known to parse on the backend.
 */
function keyToken(code: string, key: string): string | null {
    const letter = /^Key([A-Z])$/.exec(code);
    if (letter) return letter[1];
    const digit = /^Digit([0-9])$/.exec(code);
    if (digit) return digit[1];
    if (/^F([1-9]|1[0-2])$/.test(code)) return code;
    if (code === 'Space') return 'Space';
    // Bare modifier presses are not a real key.
    if (
        code.startsWith('Control') ||
        code.startsWith('Alt') ||
        code.startsWith('Shift') ||
        code.startsWith('Meta')
    ) {
        return null;
    }
    // Last-resort: a single printable character (e.g. layouts where `code` is unusual).
    if (key && key.length === 1 && /[A-Za-z0-9]/.test(key)) return key.toUpperCase();
    return null;
}

/**
 * Build an accelerator string from a keydown, or `null` if it is not a usable combo.
 * Requires at least one modifier (a bare letter would shadow normal typing). Modifier
 * order is fixed (Control, Alt, Shift, Command) to match the Rust default and keep the
 * output deterministic regardless of parser ordering.
 */
export function accelFromEvent(e: CapturedKey): string | null {
    const key = keyToken(e.code, e.key);
    if (key === null) return null;

    const mods: string[] = [];
    if (e.ctrlKey) mods.push('Control');
    if (e.altKey) mods.push('Alt');
    if (e.shiftKey) mods.push('Shift');
    if (e.metaKey) mods.push('Command');
    if (mods.length === 0) return null;

    return [...mods, key].join('+');
}

export type ShortcutRecorderProps = {
    /** Current accelerator, shown until a new combo is captured. */
    current: string;
    /** Called with the captured accelerator when the user confirms. */
    onConfirm: (accel: string) => void;
    /** Called on Cancel or Escape. */
    onCancel: () => void;
};

export function ShortcutRecorder({ current, onConfirm, onCancel }: ShortcutRecorderProps) {
    const [captured, setCaptured] = useState<string | null>(null);

    useEffect(() => {
        const onKey = (e: KeyboardEvent) => {
            // Always swallow the key while recording so the calculator never reacts.
            e.preventDefault();
            e.stopPropagation();
            if (e.key === 'Escape') {
                onCancel();
                return;
            }
            const accel = accelFromEvent(e);
            if (accel) setCaptured(accel);
        };
        // Capture phase: fire before (and block) the bubble-phase window keydown handler in App.tsx.
        window.addEventListener('keydown', onKey, true);
        return () => window.removeEventListener('keydown', onKey, true);
    }, [onCancel]);

    const shown = captured ?? current;

    return createPortal(
        <div className="shortcut-recorder-backdrop" role="dialog" aria-label="Record global shortcut">
            <div className="shortcut-recorder">
                <h3 className="shortcut-recorder-title">Set Global Shortcut</h3>
                <p className="shortcut-recorder-hint">
                    Press a key with at least one modifier (⌃ ⌥ ⇧ ⌘).
                </p>
                <div className="shortcut-recorder-preview" aria-live="polite">
                    {formatAccel(shown) || '—'}
                </div>
                <div className="shortcut-recorder-actions">
                    <button className="settings-action-btn" onClick={onCancel}>
                        Cancel
                    </button>
                    <button
                        className="settings-action-btn"
                        disabled={!captured}
                        onClick={() => captured && onConfirm(captured)}
                    >
                        Save
                    </button>
                </div>
            </div>
        </div>,
        document.body,
    );
}

export default ShortcutRecorder;
