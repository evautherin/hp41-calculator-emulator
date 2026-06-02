// Fixed design size of the HP-41 calculator layout. The whole UI is authored
// against this size. DESIGN_WIDTH/DESIGN_HEIGHT hug the real rendered footprint
// (keyboard 392×668 + ~213px header); the outer ScaledApp wrapper fills the
// viewport by scaling up (to MAX_SCALE) or down (below 1) uniformly.
// See docs/superpowers/specs/2026-05-29-macos-menu-bar-mode-design.md.
export const DESIGN_WIDTH = 392;
export const DESIGN_HEIGHT = 900;

/**
 * Maximum upscale factor applied by computeScale. Prevents the calculator from
 * becoming absurdly large on 4K or ultra-wide displays while still filling most
 * normal windows.
 */
export const MAX_SCALE = 2;

/**
 * Uniform scale factor to fit the DESIGN_WIDTH x DESIGN_HEIGHT layout inside the
 * given viewport. Upscales up to MAX_SCALE to fill larger viewports (aspect
 * preserved, no distortion). Downscales below 1 when the viewport is smaller than
 * the design box (e.g. macOS menu-bar popover). Falls back to 1 for non-positive
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
    return Math.min(MAX_SCALE, viewportWidth / designWidth, viewportHeight / designHeight);
}
