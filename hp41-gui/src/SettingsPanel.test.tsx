// Phase 48 Plan 03 — Vitest unit tests for SettingsPanel (D-48.1 / D-48.2 / D-48.4).
//
// Tests cover:
//   1. Null render when open=false
//   2. Renders 4 theme radio buttons when open=true
//   3. Dark radio is checked when currentTheme is "dark"
//   4. Calls onThemeChange when a radio is clicked
//   5. Displays correct theme labels (Dark, Light, Classic Beige, High Contrast)
//   6. Has correct accessibility attributes (role="dialog", aria-label="Settings")
//
// Tauri invoke is mocked via vi.mock('@tauri-apps/api/core') following the
// HelpOverlay.test.tsx pattern.

import { describe, it, expect, vi } from 'vitest';
import { render, fireEvent } from '@testing-library/react';
import { SettingsPanel } from './SettingsPanel';

// Mock Tauri API so tests run in the browser-less Vitest environment.
vi.mock('@tauri-apps/api/core', () => ({
    invoke: vi.fn().mockResolvedValue(undefined),
}));

describe('SettingsPanel', () => {
    it('renders null when open=false', () => {
        const { container } = render(
            <SettingsPanel
                open={false}
                onClose={() => {}}
                currentTheme="dark"
                onThemeChange={() => {}}
            />
        );
        expect(container.firstChild).toBeNull();
    });

    it('renders 4 theme radio buttons when open=true', () => {
        const { container } = render(
            <SettingsPanel
                open={true}
                onClose={() => {}}
                currentTheme="dark"
                onThemeChange={() => {}}
            />
        );
        const radios = container.querySelectorAll('input[type="radio"]');
        expect(radios.length).toBe(4);
    });

    it('dark radio is checked when currentTheme is dark', () => {
        const { container } = render(
            <SettingsPanel
                open={true}
                onClose={() => {}}
                currentTheme="dark"
                onThemeChange={() => {}}
            />
        );
        const darkRadio = container.querySelector('input[type="radio"][value="dark"]') as HTMLInputElement;
        expect(darkRadio).not.toBeNull();
        expect(darkRadio.checked).toBe(true);
    });

    it('calls onThemeChange when a radio is clicked', () => {
        const mockFn = vi.fn();
        const { container } = render(
            <SettingsPanel
                open={true}
                onClose={() => {}}
                currentTheme="dark"
                onThemeChange={mockFn}
            />
        );
        const lightRadio = container.querySelector('input[type="radio"][value="light"]') as HTMLInputElement;
        expect(lightRadio).not.toBeNull();
        fireEvent.click(lightRadio);
        expect(mockFn).toHaveBeenCalledWith('light');
    });

    it('displays correct theme labels', () => {
        const { container } = render(
            <SettingsPanel
                open={true}
                onClose={() => {}}
                currentTheme="dark"
                onThemeChange={() => {}}
            />
        );
        const text = container.textContent ?? '';
        expect(text).toContain('Dark');
        expect(text).toContain('Light');
        expect(text).toContain('Classic Beige');
        expect(text).toContain('High Contrast');
    });

    it('has correct accessibility attributes', () => {
        const { container } = render(
            <SettingsPanel
                open={true}
                onClose={() => {}}
                currentTheme="dark"
                onThemeChange={() => {}}
            />
        );
        const panel = container.querySelector('[role="dialog"]') as HTMLElement;
        expect(panel).not.toBeNull();
        expect(panel.getAttribute('aria-label')).toBe('Settings');
    });
});
