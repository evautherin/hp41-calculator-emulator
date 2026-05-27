// Phase 49 Plan 04 — OnboardingWizard stub for TypeScript compilation.
//
// This stub provides the exact interface that App.tsx consumes (Task 1 of Plan 04).
// The full implementation is created by Plan 49-03 (OnboardingWizard component with
// 5-panel navigation, keyboard shortcuts section, and expandable function entries).
//
// The stub is intentional and minimal — it exports the correct props type so App.tsx
// can import and use OnboardingWizard without TypeScript errors. Plan 49-03's full
// implementation will replace this file during the wave 3 merge.
//
// Contract (from 49-04-PLAN.md interfaces section, mirrors Plan 49-03 must_haves):
//   open: boolean    — controls visibility
//   onClose: () => void — called when wizard is dismissed
//   isFirstRun: boolean — controls Esc behavior (first-run blocks Esc; re-open allows it)

import { useEffect } from 'react';

export type OnboardingWizardProps = {
    open: boolean;
    onClose: () => void;
    isFirstRun: boolean;
};

export function OnboardingWizard({ open, onClose, isFirstRun }: OnboardingWizardProps) {
    // Esc closes wizard only in re-open mode (not first-run), per D-49.9.
    useEffect(() => {
        if (!open) return;
        const onKey = (e: KeyboardEvent) => {
            if (e.key === 'Escape' && !isFirstRun) {
                e.preventDefault();
                onClose();
            }
        };
        window.addEventListener('keydown', onKey);
        return () => window.removeEventListener('keydown', onKey);
    }, [open, onClose, isFirstRun]);

    if (!open) return null;

    // Stub: renders a minimal placeholder.
    // Full implementation (5-panel wizard with RPN tutorial content) provided by Plan 49-03.
    return (
        <div className="onboarding-overlay" role="dialog" aria-label="Quick Start Guide">
            <div className="onboarding-content">
                <p>Quick Start Guide (loading...)</p>
                {!isFirstRun && (
                    <button onClick={onClose}>Close</button>
                )}
            </div>
        </div>
    );
}

export default OnboardingWizard;
