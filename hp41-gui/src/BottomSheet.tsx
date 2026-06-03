// Phase 55 Plan 05 — BottomSheet component (TOUCH-09)
//
// Pull-up bottom sheet for iOS print log and PRGM listing.
// Tap the drag handle / header to toggle between collapsed (peek) and expanded.
// Returns null when visible=false — App.tsx gates mounting with isIos.
//
// Props:
//   id        — HTML id for the sheet root ("print-sheet" | "prgm-sheet")
//   title     — Sheet header label ("PRINT LOG" | "PROGRAM")
//   visible   — Controls whether the sheet is mounted
//   emptyText — Fallback copy when no children (null/undefined children)
//   children  — Scrollable content (print lines, program steps)

import { useState } from 'react';

interface BottomSheetProps {
  id: string;
  title: string;
  visible: boolean;
  emptyText?: string;
  children?: React.ReactNode;
}

export default function BottomSheet({ id, title, visible, emptyText, children }: BottomSheetProps) {
  const [expanded, setExpanded] = useState(false);

  // Early return when not visible — keeps the DOM clean on desktop (isIos=false).
  if (!visible) return null;

  // Determine whether children are non-empty.
  // React.Children.count handles null/undefined/array safely.
  const hasChildren =
    children !== null &&
    children !== undefined &&
    (typeof children !== 'boolean');

  return (
    <div id={id} className={`bottom-sheet${expanded ? ' expanded' : ''}`} role="dialog" aria-label={title}>
      <div className="bottom-sheet-header">
        <span className="bottom-sheet-title">{title}</span>
        <button
          className="bottom-sheet-handle"
          aria-label={expanded ? 'Collapse panel' : 'Expand panel'}
          onClick={() => setExpanded(e => !e)}
          style={{ touchAction: 'manipulation', WebkitTapHighlightColor: 'transparent' } as React.CSSProperties}
        >
          {expanded ? '▼' : '▲'}
        </button>
      </div>
      <div
        className="bottom-sheet-content"
        style={{ WebkitOverflowScrolling: 'touch' } as React.CSSProperties}
      >
        {hasChildren ? (
          children
        ) : emptyText ? (
          <div className="bottom-sheet-empty">{emptyText}</div>
        ) : null}
      </div>
    </div>
  );
}
