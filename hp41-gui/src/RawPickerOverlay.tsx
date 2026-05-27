// Phase 50 Plan 03 — RawPickerOverlay component (D-50.4 / D-50.5 / D-50.6).
//
// Multi-program picker modal for selecting which programs to import from a
// .raw archive. Follows the SettingsPanel/HelpOverlay overlay pattern:
//   - Full-cover overlay anchored to `.calculator` (position: absolute, z-index: 60)
//   - role="dialog", aria-label="Select Programs", aria-modal="true"
//   - Escape key dismisses (matches existing overlay dismiss pattern)
//   - On mount, focus the first checkbox for accessibility per UI-SPEC
//   - "Import Selected (N)" primary button disabled when no checkboxes checked
//   - "Cancel Import" secondary button calls onClose (silent dismiss, no toast)
//
// Props:
//   programs: Array<{ label: string; index: number; byte_len: number }>
//   onConfirm: (selectedIndices: number[]) => void
//   onClose: () => void
//
// CSS classes (defined in App.css):
//   .raw-picker-overlay, .raw-picker-header, .raw-picker-heading,
//   .raw-picker-close, .raw-picker-list, .raw-picker-row,
//   .raw-picker-name, .raw-picker-size, .raw-picker-footer

import { useState, useEffect, useRef } from 'react';

export interface ProgramEntry {
  label: string;
  index: number;
  byte_len: number;
}

export interface RawPickerOverlayProps {
  programs: ProgramEntry[];
  onConfirm: (selectedIndices: number[]) => void;
  onClose: () => void;
}

export function RawPickerOverlay({ programs, onConfirm, onClose }: RawPickerOverlayProps) {
  // Multi-select state: set of selected program indices (not array positions)
  const [selected, setSelected] = useState<Set<number>>(new Set());
  const firstCheckboxRef = useRef<HTMLInputElement>(null);

  // Escape key dismisses the picker (D-50.5 accessibility; matches existing pattern)
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        e.preventDefault();
        onClose();
      }
    };
    window.addEventListener('keydown', handler);
    return () => window.removeEventListener('keydown', handler);
  }, [onClose]);

  // On mount, focus the first checkbox for accessibility per UI-SPEC
  useEffect(() => {
    if (firstCheckboxRef.current) {
      firstCheckboxRef.current.focus();
    }
  }, []);

  function toggleProgram(index: number) {
    setSelected(prev => {
      const next = new Set(prev);
      if (next.has(index)) {
        next.delete(index);
      } else {
        next.add(index);
      }
      return next;
    });
  }

  function handleImport() {
    if (selected.size === 0) return;
    onConfirm(Array.from(selected));
  }

  const selectedCount = selected.size;

  return (
    <div
      className="raw-picker-overlay"
      role="dialog"
      aria-label="Select Programs"
      aria-modal="true"
    >
      {/* Header bar: heading + count chip + close button */}
      <div className="raw-picker-header">
        <h3 className="raw-picker-heading">SELECT PROGRAMS</h3>
        <span className="raw-picker-count">{programs.length} program{programs.length === 1 ? '' : 's'}</span>
        <button
          className="raw-picker-close"
          aria-label="Close picker"
          onClick={onClose}
        >
          ×
        </button>
      </div>

      {/* Scrollable list of programs */}
      <div className="raw-picker-list">
        {programs.length === 0 ? (
          <p className="raw-picker-empty">No programs found in this file.</p>
        ) : (
          programs.map((prog, pos) => {
            const checkId = `raw-picker-cb-${prog.index}`;
            return (
              <div key={prog.index} className="raw-picker-row">
                <input
                  type="checkbox"
                  id={checkId}
                  ref={pos === 0 ? firstCheckboxRef : undefined}
                  checked={selected.has(prog.index)}
                  onChange={() => toggleProgram(prog.index)}
                />
                <label className="raw-picker-name" htmlFor={checkId}>
                  {prog.label}
                </label>
                <span className="raw-picker-size">{prog.byte_len} B</span>
              </div>
            );
          })
        )}
      </div>

      {/* Footer: Import Selected (N) primary + Cancel Import secondary */}
      <div className="raw-picker-footer">
        <button
          className="raw-picker-btn-secondary"
          onClick={onClose}
        >
          Cancel Import
        </button>
        <button
          className="raw-picker-btn-primary"
          disabled={selectedCount === 0}
          onClick={handleImport}
          aria-disabled={selectedCount === 0}
        >
          Import Selected ({selectedCount})
        </button>
      </div>
    </div>
  );
}

export default RawPickerOverlay;
