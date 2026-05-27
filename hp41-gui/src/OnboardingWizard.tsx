// Phase 49 Plan 03 — OnboardingWizard component (ONBOARD-01 / D-49.1 / D-49.2 / D-49.3).
//
// 5-panel quick-start guide overlay for HP-41C RPN concepts.
// Follows the HelpOverlay overlay pattern exactly (D-49.3):
//   - Full-cover overlay anchored to `.calculator` (position: absolute, z-index: 60)
//   - Early-return null when closed
//   - Esc handler via useEffect — fires only on re-open (isFirstRun=false)
//   - Reset-on-open: panelIdx resets to 0 on each open (D-49.9)
//   - role="dialog", aria-label="HP-41C Quick Start Guide", aria-modal="true"
//
// First-run behavior: Esc and click-outside do NOT dismiss (user must Skip or Finish).
// Re-open behavior: Esc and click-outside DO dismiss (matches existing overlay pattern).

import { useState, useEffect } from 'react';

export type OnboardingWizardProps = {
    open: boolean;
    onClose: () => void;
    isFirstRun: boolean;
};

// ---------------------------------------------------------------------------
// Panel content
// ---------------------------------------------------------------------------

interface PanelDef {
    title: string;
    subhead: string;
    content: React.ReactNode;
}

const PANELS: PanelDef[] = [
    {
        title: 'Welcome to HP-41C',
        subhead: 'An RPN calculator — numbers first, operator second.',
        content: (
            <div>
                <p>In RPN (Reverse Polish Notation), you enter operands before the operator. To add 3 and 4:</p>
                <ol style={{ marginTop: 8, paddingLeft: 20 }}>
                    <li>Type <strong>3</strong></li>
                    <li>Press <strong>Enter</strong> (to push 3 onto the stack)</li>
                    <li>Type <strong>4</strong></li>
                    <li>Press <strong>+</strong></li>
                </ol>
                <p style={{ marginTop: 8 }}>Result: <strong>7</strong> appears in the display.</p>
                <pre className="wizard-stack-diagram">{`Before +         After +
  T: 0.0000        T: 0.0000
  Z: 0.0000        Z: 0.0000
  Y: 3.0000   →    Y: 0.0000
  X: 4.0000        X: 7.0000`}</pre>
                <p style={{ marginTop: 8 }}>No parentheses, no equals key — just stack operations.</p>
            </div>
        ),
    },
    {
        title: 'The Four-Level Stack',
        subhead: 'X, Y, Z, T — the calculator\'s working memory.',
        content: (
            <div>
                <p>The HP-41C has a 4-register stack. When you press <strong>Enter</strong>, the value in X is pushed up through Y → Z → T, and the current value is duplicated in X.</p>
                <pre className="wizard-stack-diagram">{`Stack after pressing Enter:
  T: 0.0000  ←  Z was here
  Z: 0.0000  ←  Y was here
  Y: 3.0000  ←  X was copied up
  X: 3.0000  ←  entry continues here`}</pre>
                <p style={{ marginTop: 8 }}>When an arithmetic operation runs, it consumes X and Y, replacing X with the result. Z and T drop down to fill the gap. T is replicated (not lost).</p>
                <p style={{ marginTop: 8 }}>The <strong>X register</strong> is always what you see in the display. Use <strong>r</strong> to roll the stack down, or <strong>x</strong> to swap X and Y.</p>
            </div>
        ),
    },
    {
        title: 'SHIFT & Function Access',
        subhead: 'Tab arms SHIFT. ? opens the function list. XEQ runs by name.',
        content: (
            <div>
                <p><strong>One-shot SHIFT (Tab key):</strong> Press <strong>Tab</strong> once to arm the SHIFT prefix. The next key you press dispatches its shifted variant. SHIFT auto-disarms after one key.</p>
                <p style={{ marginTop: 8 }}><strong>? overlay:</strong> Press <strong>?</strong> to open the full function reference. Search by name, category, or description. The first section lists keyboard shortcuts.</p>
                <p style={{ marginTop: 8 }}><strong>XEQ by name:</strong> Press <strong>\</strong> (backslash) to open an XEQ prompt. Type a function name (e.g. SINH) and press Enter to execute it. Use this for functions without a direct key.</p>
                <p style={{ marginTop: 8 }}><strong>ALPHA mode:</strong> Press <strong>A</strong> to enter alpha input for the display. ALPHA overrides SHIFT when active.</p>
            </div>
        ),
    },
    {
        title: 'Keyboard Shortcuts',
        subhead: 'The top 10 shortcuts — full list in the ? reference.',
        content: (
            <div>
                <p style={{ marginBottom: 8 }}>The most useful keyboard shortcuts for daily use:</p>
                <table className="wizard-shortcut-table">
                    <tbody>
                        <tr className="wizard-shortcut-row">
                            <td className="wizard-shortcut-key">Enter</td>
                            <td className="wizard-shortcut-fn">ENTER — push value onto stack</td>
                        </tr>
                        <tr className="wizard-shortcut-row">
                            <td className="wizard-shortcut-key">Backspace</td>
                            <td className="wizard-shortcut-fn">CLX — clear X register</td>
                        </tr>
                        <tr className="wizard-shortcut-row">
                            <td className="wizard-shortcut-key">Tab</td>
                            <td className="wizard-shortcut-fn">SHIFT — arm one-shot shift prefix</td>
                        </tr>
                        <tr className="wizard-shortcut-row">
                            <td className="wizard-shortcut-key">+  -  *  /</td>
                            <td className="wizard-shortcut-fn">Arithmetic operators</td>
                        </tr>
                        <tr className="wizard-shortcut-row">
                            <td className="wizard-shortcut-key">r</td>
                            <td className="wizard-shortcut-fn">R↓ — roll stack down</td>
                        </tr>
                        <tr className="wizard-shortcut-row">
                            <td className="wizard-shortcut-key">x</td>
                            <td className="wizard-shortcut-fn">X&lt;&gt;Y — swap X and Y</td>
                        </tr>
                        <tr className="wizard-shortcut-row">
                            <td className="wizard-shortcut-key">F7 / F8</td>
                            <td className="wizard-shortcut-fn">SST / BST — single step forward/back in program</td>
                        </tr>
                        <tr className="wizard-shortcut-row">
                            <td className="wizard-shortcut-key">p</td>
                            <td className="wizard-shortcut-fn">PRGM — toggle program mode</td>
                        </tr>
                        <tr className="wizard-shortcut-row">
                            <td className="wizard-shortcut-key">Ctrl+S / F5</td>
                            <td className="wizard-shortcut-fn">SAVE — save calculator state</td>
                        </tr>
                        <tr className="wizard-shortcut-row">
                            <td className="wizard-shortcut-key">?</td>
                            <td className="wizard-shortcut-fn">Open function reference overlay</td>
                        </tr>
                    </tbody>
                </table>
            </div>
        ),
    },
    {
        title: 'Programming & XEQ',
        subhead: 'Press R/S to run a program. F5 saves your session.',
        content: (
            <div>
                <p><strong>Program mode:</strong> Press <strong>p</strong> to toggle PRGM mode. In PRGM mode, each keypress records a program step instead of executing immediately.</p>
                <p style={{ marginTop: 8 }}><strong>Labels:</strong> Use <strong>LBL</strong> to mark entry points. Programs run from LBL to RTN. XEQ by name launches a labeled routine.</p>
                <p style={{ marginTop: 8 }}><strong>R/S key:</strong> Press <strong>R/S</strong> (or the R/S button on the keyboard) to start or stop program execution. A running program can be interrupted with R/S.</p>
                <p style={{ marginTop: 8 }}><strong>SST / BST:</strong> Use <strong>F7</strong> (Single Step) and <strong>F8</strong> (Back Step) to step through a program one instruction at a time.</p>
                <p style={{ marginTop: 8 }}><strong>Save your work:</strong> Press <strong>Ctrl+S</strong> or <strong>F5</strong> to save your calculator state at any time. The calculator also auto-saves every 30 seconds to <code>~/.hp41/autosave.json</code>.</p>
            </div>
        ),
    },
];

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

export function OnboardingWizard({ open, onClose, isFirstRun }: OnboardingWizardProps) {
    const [panelIdx, setPanelIdx] = useState(0);

    // D-49.9: Reset to panel 1 on each open.
    useEffect(() => {
        if (open) {
            setPanelIdx(0);
        }
    }, [open]);

    // Esc-close: only fires when isFirstRun=false (re-open from settings).
    // On first-run, user must navigate to panel 5 or click Skip (D-49.3 / UI-SPEC).
    useEffect(() => {
        if (!open || isFirstRun) return;
        const onKey = (e: KeyboardEvent) => {
            if (e.key === 'Escape') {
                e.preventDefault();
                onClose();
            }
        };
        window.addEventListener('keydown', onKey);
        return () => window.removeEventListener('keydown', onKey);
    }, [open, isFirstRun, onClose]);

    // Early-return pattern (HelpOverlay.tsx line 183).
    if (!open) return null;

    const panel = PANELS[panelIdx];
    const isLastPanel = panelIdx === PANELS.length - 1;
    const isFirstPanel = panelIdx === 0;

    const handleBack = () => {
        if (!isFirstPanel) setPanelIdx(prev => prev - 1);
    };

    const handleNext = () => {
        if (!isLastPanel) setPanelIdx(prev => prev + 1);
    };

    const handleFinish = () => {
        onClose();
    };

    return (
        <div
            className="wizard-overlay"
            role="dialog"
            aria-label="HP-41C Quick Start Guide"
            aria-modal="true"
        >
            <div className="wizard-panel">
                <div className="wizard-panel-header">
                    <span>{panel.title}</span>
                    <span
                        className="wizard-panel-counter"
                        aria-label={`Step ${panelIdx + 1} of ${PANELS.length}`}
                    >
                        {panelIdx + 1} of {PANELS.length}
                    </span>
                </div>
                <div className="wizard-panel-body">
                    <p style={{ marginBottom: 12, fontStyle: 'italic', color: 'var(--text-muted)' }}>
                        {panel.subhead}
                    </p>
                    {panel.content}
                </div>
                <div className="wizard-panel-nav">
                    <div style={{ display: 'flex', gap: 8 }}>
                        <button
                            className="wizard-nav-btn"
                            onClick={handleBack}
                            disabled={isFirstPanel}
                            aria-disabled={isFirstPanel ? 'true' : 'false'}
                            style={{ opacity: isFirstPanel ? 0.4 : 1 }}
                        >
                            Back
                        </button>
                        {!isLastPanel ? (
                            <button
                                className="wizard-nav-btn wizard-nav-btn-primary"
                                onClick={handleNext}
                            >
                                Next
                            </button>
                        ) : (
                            <button
                                className="wizard-nav-btn wizard-nav-btn-primary"
                                onClick={handleFinish}
                            >
                                Start Using HP-41C
                            </button>
                        )}
                    </div>
                    {isFirstRun && (
                        <button
                            className="wizard-dismiss-link"
                            onClick={handleFinish}
                        >
                            Skip for now
                        </button>
                    )}
                </div>
            </div>
        </div>
    );
}

export default OnboardingWizard;
