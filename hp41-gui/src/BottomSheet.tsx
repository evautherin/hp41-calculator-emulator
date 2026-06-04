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
//   emptyText — Reserved empty-state copy. Accepted for call-site clarity but
//               NOT rendered: both call sites always pass a non-empty children
//               array (PRGM steps; print lines + a printEndRef sentinel <div>),
//               so an empty-state branch was unreachable dead code (WR-02).
//   children  — Scrollable content (print lines, program steps)

import { useState } from 'react';

interface BottomSheetProps {
  id: string;
  title: string;
  visible: boolean;
  emptyText?: string;
  children?: React.ReactNode;
}

export default function BottomSheet({ id, title, visible, children }: BottomSheetProps) {
  const [expanded, setExpanded] = useState(false);

  // Early return when not visible — keeps the DOM clean on desktop (isIos=false).
  if (!visible) return null;

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
        {children}
      </div>
    </div>
  );
}
