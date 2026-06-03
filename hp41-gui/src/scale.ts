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
 *
 * Division of responsibility:
 *   - `reservedHeight` is for UI chrome that sits ON TOP OF the available area at
 *     the bottom (e.g. the BottomSheet collapsed peek at 32px). The CALLER
 *     (ScaledApp in main.tsx) passes the peek height here so the scaler fits the
 *     content above the sheet without knowing about safe-area insets.
 *   - Available-area reduction for safe-area insets (Dynamic Island, home indicator)
 *     is applied by the CALLER by passing a smaller viewportWidth/viewportHeight
 *     (the outer wrapper's content-box after env() padding). computeScale itself
 *     only needs the reservedHeight knob for bottom-sheet peek reservation.
 */
export function computeScale(
    viewportWidth: number,
    viewportHeight: number,
    designWidth: number = DESIGN_WIDTH,
    designHeight: number = DESIGN_HEIGHT,
    reservedHeight: number = 0,
): number {
    if (viewportWidth <= 0 || viewportHeight <= 0) {
        return 1;
    }
    const effectiveHeight = viewportHeight - reservedHeight;
    if (effectiveHeight <= 0) {
        return 1;
    }
    return Math.min(MAX_SCALE, viewportWidth / designWidth, effectiveHeight / designHeight);
}
