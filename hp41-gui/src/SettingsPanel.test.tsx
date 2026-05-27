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
// Phase 49 Plan 04 — Extended tests for Quick Start section (D-49.8 / D-49.9):
//   7. Renders Quick Start section with "Show Guide" button
//   8. Clicking "Show Guide" calls onClose then onShowOnboarding
//   (All existing tests updated with required onShowOnboarding prop)
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
                onShowOnboarding={vi.fn()}
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
                onShowOnboarding={vi.fn()}
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
                onShowOnboarding={vi.fn()}
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
                onShowOnboarding={vi.fn()}
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
                onShowOnboarding={vi.fn()}
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
                onShowOnboarding={vi.fn()}
            />
        );
        const panel = container.querySelector('[role="dialog"]') as HTMLElement;
        expect(panel).not.toBeNull();
        expect(panel.getAttribute('aria-label')).toBe('Settings');
    });

    // Phase 49 Plan 04 — Quick Start section tests (D-49.8 / D-49.9)

    it('renders Quick Start section with "Show Guide" button', () => {
        const { container } = render(
            <SettingsPanel
                open={true}
                onClose={() => {}}
                currentTheme="dark"
                onThemeChange={() => {}}
                onShowOnboarding={vi.fn()}
            />
        );
        const text = container.textContent ?? '';
        expect(text).toContain('Quick Start');
        const btn = container.querySelector('button.settings-action-btn') as HTMLButtonElement;
        expect(btn).not.toBeNull();
        expect(btn.textContent).toBe('Show Guide');
    });

    it('clicking "Show Guide" calls onClose then onShowOnboarding', () => {
        const onClose = vi.fn();
        const onShowOnboarding = vi.fn();
        const { container } = render(
            <SettingsPanel
                open={true}
                onClose={onClose}
                currentTheme="dark"
                onThemeChange={() => {}}
                onShowOnboarding={onShowOnboarding}
            />
        );
        const btn = container.querySelector('button.settings-action-btn') as HTMLButtonElement;
        expect(btn).not.toBeNull();
        fireEvent.click(btn);
        expect(onClose).toHaveBeenCalledOnce();
        expect(onShowOnboarding).toHaveBeenCalledOnce();
    });
});
