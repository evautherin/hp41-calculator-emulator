// Phase 49 Plan 03 — Vitest unit tests for OnboardingWizard (ONBOARD-01 / D-49.1).
//
// Tests cover:
//   1. Null render when open=false
//   2. Renders wizard with panel 1 when open=true
//   3. Shows panel counter "1 of 5"
//   4. Next button advances to panel 2
//   5. Back button returns to panel 1
//   6. Back button disabled on panel 1
//   7. Panel 5 shows "Start Using HP-41C" button
//   8. Clicking "Start Using HP-41C" calls onClose
//   9. Esc does NOT close on first-run
//  10. Esc closes on re-open
//  11. Resets to panel 1 on re-open
//
// Mock pattern from SettingsPanel.test.tsx.

import { describe, it, expect, vi } from 'vitest';
import { render, fireEvent } from '@testing-library/react';
import { OnboardingWizard } from './OnboardingWizard';

// Mock Tauri API so tests run in the browser-less Vitest environment.
vi.mock('@tauri-apps/api/core', () => ({
    invoke: vi.fn().mockResolvedValue(undefined),
}));

describe('OnboardingWizard', () => {
    it('renders null when open=false', () => {
        const { container } = render(
            <OnboardingWizard open={false} onClose={() => {}} isFirstRun={true} />
        );
        expect(container.firstChild).toBeNull();
    });

    it('renders wizard with panel 1 when open=true', () => {
        const { container } = render(
            <OnboardingWizard open={true} onClose={() => {}} isFirstRun={true} />
        );
        expect(container.querySelector('.wizard-overlay')).not.toBeNull();
        expect(container.textContent).toContain('Welcome to HP-41C');
    });

    it('shows panel counter "1 of 5"', () => {
        const { container } = render(
            <OnboardingWizard open={true} onClose={() => {}} isFirstRun={true} />
        );
        const counter = container.querySelector('.wizard-panel-counter');
        expect(counter).not.toBeNull();
        expect(counter!.textContent).toContain('1 of 5');
    });

    it('Next button advances to panel 2', () => {
        const { container } = render(
            <OnboardingWizard open={true} onClose={() => {}} isFirstRun={true} />
        );
        const nextBtn = Array.from(container.querySelectorAll('button')).find(
            b => b.textContent?.trim() === 'Next'
        ) as HTMLButtonElement | undefined;
        expect(nextBtn, 'Next button must exist on panel 1').toBeTruthy();
        fireEvent.click(nextBtn!);
        expect(container.textContent).toContain('The Four-Level Stack');
    });

    it('Back button returns to panel 1', () => {
        const { container } = render(
            <OnboardingWizard open={true} onClose={() => {}} isFirstRun={true} />
        );
        // Advance to panel 2.
        const nextBtn = Array.from(container.querySelectorAll('button')).find(
            b => b.textContent?.trim() === 'Next'
        ) as HTMLButtonElement;
        fireEvent.click(nextBtn);
        expect(container.textContent).toContain('The Four-Level Stack');

        // Go back to panel 1.
        const backBtn = Array.from(container.querySelectorAll('button')).find(
            b => b.textContent?.trim() === 'Back'
        ) as HTMLButtonElement;
        expect(backBtn).toBeTruthy();
        fireEvent.click(backBtn);
        expect(container.textContent).toContain('Welcome to HP-41C');
    });

    it('Back button is disabled on panel 1', () => {
        const { container } = render(
            <OnboardingWizard open={true} onClose={() => {}} isFirstRun={true} />
        );
        const backBtn = Array.from(container.querySelectorAll('button')).find(
            b => b.textContent?.trim() === 'Back'
        ) as HTMLButtonElement | undefined;
        expect(backBtn, 'Back button must exist on panel 1').toBeTruthy();
        expect(backBtn!.disabled).toBe(true);
    });

    it('panel 5 shows "Start Using HP-41C" button', () => {
        const { container } = render(
            <OnboardingWizard open={true} onClose={() => {}} isFirstRun={true} />
        );
        // Advance through 4 "Next" clicks to reach panel 5.
        for (let i = 0; i < 4; i++) {
            const nextBtn = Array.from(container.querySelectorAll('button')).find(
                b => b.textContent?.trim() === 'Next'
            ) as HTMLButtonElement;
            expect(nextBtn, `Next button must exist on panel ${i + 1}`).toBeTruthy();
            fireEvent.click(nextBtn);
        }
        // Panel 5 should show "Start Using HP-41C" instead of "Next".
        const finishBtn = Array.from(container.querySelectorAll('button')).find(
            b => b.textContent?.trim() === 'Start Using HP-41C'
        ) as HTMLButtonElement | undefined;
        expect(finishBtn, '"Start Using HP-41C" button must exist on panel 5').toBeTruthy();
    });

    it('clicking "Start Using HP-41C" calls onClose', () => {
        const onClose = vi.fn();
        const { container } = render(
            <OnboardingWizard open={true} onClose={onClose} isFirstRun={true} />
        );
        // Advance to panel 5.
        for (let i = 0; i < 4; i++) {
            const nextBtn = Array.from(container.querySelectorAll('button')).find(
                b => b.textContent?.trim() === 'Next'
            ) as HTMLButtonElement;
            fireEvent.click(nextBtn);
        }
        const finishBtn = Array.from(container.querySelectorAll('button')).find(
            b => b.textContent?.trim() === 'Start Using HP-41C'
        ) as HTMLButtonElement;
        expect(finishBtn).toBeTruthy();
        fireEvent.click(finishBtn);
        expect(onClose).toHaveBeenCalledOnce();
    });

    it('Esc does NOT close on first-run (isFirstRun=true)', () => {
        const onClose = vi.fn();
        render(
            <OnboardingWizard open={true} onClose={onClose} isFirstRun={true} />
        );
        fireEvent.keyDown(window, { key: 'Escape' });
        expect(onClose).not.toHaveBeenCalled();
    });

    it('Esc closes on re-open (isFirstRun=false)', () => {
        const onClose = vi.fn();
        render(
            <OnboardingWizard open={true} onClose={onClose} isFirstRun={false} />
        );
        fireEvent.keyDown(window, { key: 'Escape' });
        expect(onClose).toHaveBeenCalledOnce();
    });

    it('resets to panel 1 on re-open (D-49.9)', () => {
        const { container, rerender } = render(
            <OnboardingWizard open={true} onClose={() => {}} isFirstRun={false} />
        );
        // Advance to panel 3.
        for (let i = 0; i < 2; i++) {
            const nextBtn = Array.from(container.querySelectorAll('button')).find(
                b => b.textContent?.trim() === 'Next'
            ) as HTMLButtonElement;
            fireEvent.click(nextBtn);
        }
        expect(container.textContent).toContain('SHIFT & Function Access');

        // Close.
        rerender(<OnboardingWizard open={false} onClose={() => {}} isFirstRun={false} />);
        // Re-open.
        rerender(<OnboardingWizard open={true} onClose={() => {}} isFirstRun={false} />);

        // Should be back at panel 1.
        expect(container.textContent).toContain('Welcome to HP-41C');
        const counter = container.querySelector('.wizard-panel-counter');
        expect(counter!.textContent).toContain('1 of 5');
    });
});
