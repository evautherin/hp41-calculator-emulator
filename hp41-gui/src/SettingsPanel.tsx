// Phase 48 Plan 03 — Settings panel component (D-48.1 / D-48.2 / D-48.4).
//
// Renders a popover with theme radio buttons. Click-outside closes the panel
// (no explicit close button, per D-48.4). Returns null when closed to avoid
// DOM overhead (matches HelpOverlay early-return pattern).
//
// Phase 49 Plan 04 — Extended with Quick Start section (D-48.3 / D-49.8 / D-49.9):
// - onShowOnboarding prop added to SettingsPanelProps
// - Quick Start section with "Show Guide" button added below Theme section

import { useRef, useEffect } from 'react';

export type SettingsPanelProps = {
    open: boolean;
    onClose: () => void;
    currentTheme: string;
    onThemeChange: (theme: string) => void;
    onShowOnboarding: () => void;  // D-49.8 / D-49.9: re-open wizard from settings
};

const THEMES = [
    { id: 'dark', label: 'Dark' },
    { id: 'light', label: 'Light' },
    { id: 'classic-beige', label: 'Classic Beige' },
    { id: 'high-contrast', label: 'High Contrast' },
] as const;

export function SettingsPanel({ open, onClose, currentTheme, onThemeChange, onShowOnboarding }: SettingsPanelProps) {
    const panelRef = useRef<HTMLDivElement>(null);

    // Click-outside dismiss — only register listener when open (D-48.4).
    // Use `mousedown` (not `click`) so the gear-button's `onMouseDown` with
    // stopPropagation fires before this listener, preventing immediate re-close.
    useEffect(() => {
        if (!open) return;
        const handler = (e: MouseEvent) => {
            if (panelRef.current && !panelRef.current.contains(e.target as Node)) {
                onClose();
            }
        };
        document.addEventListener('mousedown', handler);
        return () => document.removeEventListener('mousedown', handler);
    }, [open, onClose]);

    if (!open) return null;

    return (
        <div
            ref={panelRef}
            className="settings-panel"
            role="dialog"
            aria-label="Settings"
        >
            <section className="settings-section">
                <h3 className="settings-section-heading">Theme</h3>
                {THEMES.map(t => (
                    <label key={t.id} className="settings-radio-row">
                        <input
                            type="radio"
                            name="theme"
                            value={t.id}
                            checked={currentTheme === t.id}
                            onChange={() => onThemeChange(t.id)}
                        />
                        {t.label}
                    </label>
                ))}
            </section>
            <hr className="settings-section-divider" />
            <section className="settings-section">
                <h3 className="settings-section-heading">Quick Start</h3>
                {/* D-49.9: clicking "Show Guide" closes settings first, then opens the onboarding wizard */}
                <button
                    className="settings-action-btn"
                    onClick={() => { onClose(); onShowOnboarding(); }}
                >
                    Show Guide
                </button>
            </section>
        </div>
    );
}

export default SettingsPanel;
