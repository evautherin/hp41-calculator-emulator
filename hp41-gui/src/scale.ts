// Fixed design size of the HP-41 calculator layout. The whole UI is authored
// against this size; the menu-bar popover (and any small screen) scales it down
// uniformly so nothing is clipped.
// See docs/superpowers/specs/2026-05-29-macos-menu-bar-mode-design.md.
export const DESIGN_WIDTH = 440;
export const DESIGN_HEIGHT = 1020;

/**
 * Uniform scale factor to fit the DESIGN_WIDTH x DESIGN_HEIGHT layout inside the
 * given viewport. Never upscales (capped at 1). Falls back to 1 for non-positive
 * inputs (e.g. a zero-size viewport during first paint).
 */
export function computeScale(
    viewportWidth: number,
    viewportHeight: number,
    designWidth: number = DESIGN_WIDTH,
    designHeight: number = DESIGN_HEIGHT,
): number {
    if (viewportWidth <= 0 || viewportHeight <= 0) {
        return 1;
    }
    return Math.min(1, viewportWidth / designWidth, viewportHeight / designHeight);
}
