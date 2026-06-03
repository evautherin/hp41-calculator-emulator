// Phase 55 Plan 05 — Unit tests for BottomSheet.tsx (TOUCH-09 TDD RED → GREEN)
//
// 4 behaviors:
//  1. With visible=false, BottomSheet renders null (container empty).
//  2. With visible=true, BottomSheet renders the sheet with its title and children.
//  3. Clicking the drag handle / header toggles the `expanded` class on the sheet.
//  4. Empty children shows the empty-state copy (e.g. "No print output yet." for print).

import { describe, it, expect, vi, afterEach } from 'vitest';
import { render, fireEvent, cleanup } from '@testing-library/react';
import BottomSheet from './BottomSheet';

// --- Plugin mocks (prevent module-not-found errors in Vitest resolution) ------

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn().mockResolvedValue(undefined),
}));

vi.mock('@tauri-apps/plugin-haptics', () => ({
  impactFeedback: vi.fn().mockResolvedValue(undefined),
  notificationFeedback: vi.fn().mockResolvedValue(undefined),
}));

// --- Cleanup ---------------------------------------------------------------

afterEach(cleanup);

// --- Helpers ---------------------------------------------------------------

function renderSheet(overrides: {
  id?: string;
  title?: string;
  visible?: boolean;
  children?: React.ReactNode;
  emptyText?: string;
} = {}) {
  return render(
    <BottomSheet
      id={overrides.id ?? 'test-sheet'}
      title={overrides.title ?? 'TEST SHEET'}
      visible={overrides.visible ?? true}
      emptyText={overrides.emptyText}
    >
      {overrides.children}
    </BottomSheet>,
  );
}

// --- Tests -----------------------------------------------------------------

describe('BottomSheet', () => {
  it('renders null when visible=false', () => {
    const { container } = renderSheet({ visible: false });
    expect(container.firstChild).toBeNull();
  });

  it('renders the sheet with its title and children when visible=true', () => {
    const { container } = renderSheet({
      visible: true,
      title: 'PRINT LOG',
      children: <div className="print-line">Hello</div>,
    });
    expect(container.firstChild).not.toBeNull();
    expect(container.textContent).toContain('PRINT LOG');
    expect(container.textContent).toContain('Hello');
  });

  it('toggles expanded class on the sheet when handle is clicked', () => {
    const { container } = renderSheet({ visible: true });
    const sheet = container.querySelector('.bottom-sheet') as HTMLElement;
    expect(sheet).not.toBeNull();
    // Initially collapsed — should NOT have 'expanded' class
    expect(sheet.classList.contains('expanded')).toBe(false);

    // Click the drag handle / header toggle button
    const handle = container.querySelector('.bottom-sheet-handle') as HTMLButtonElement;
    expect(handle).not.toBeNull();
    fireEvent.click(handle);

    // Now should be expanded
    expect(sheet.classList.contains('expanded')).toBe(true);

    // Click again — should collapse
    fireEvent.click(handle);
    expect(sheet.classList.contains('expanded')).toBe(false);
  });

  it('shows empty-state copy when children are empty/null and emptyText is provided', () => {
    const { container } = renderSheet({
      visible: true,
      children: undefined,
      emptyText: 'No print output yet.',
    });
    expect(container.textContent).toContain('No print output yet.');
  });
});
