import { describe, it, expect } from "vitest";
import { computeScale, MAX_SCALE, DESIGN_WIDTH, DESIGN_HEIGHT } from "./scale";

describe("computeScale — MAX_SCALE upscale-to-fill contract", () => {
    it("exports corrected design box constants", () => {
        expect(DESIGN_WIDTH).toBe(392);
        expect(DESIGN_HEIGHT).toBe(900);
    });

    it("exports MAX_SCALE ≈ 2", () => {
        expect(MAX_SCALE).toBe(2);
    });

    it("upscales to MAX_SCALE when viewport is 2× the design box", () => {
        expect(computeScale(DESIGN_WIDTH * 2, DESIGN_HEIGHT * 2)).toBe(2);
    });

    it("upscales past 1 but below the cap on a 1.5× viewport", () => {
        expect(computeScale(DESIGN_WIDTH * 1.5, DESIGN_HEIGHT * 1.5)).toBeCloseTo(1.5, 5);
    });

    it("caps at MAX_SCALE on a huge 4K display", () => {
        expect(computeScale(10000, 10000)).toBe(MAX_SCALE);
    });

    it("returns 1 at exactly the design size (neutral)", () => {
        expect(computeScale(DESIGN_WIDTH, DESIGN_HEIGHT)).toBe(1);
    });

    it("downscales below 1 when viewport height is smaller than design (menu-bar popover)", () => {
        expect(computeScale(DESIGN_WIDTH, DESIGN_HEIGHT / 2)).toBeCloseTo(0.5, 5);
    });

    it("downscales below 1 when viewport width is smaller than design", () => {
        expect(computeScale(DESIGN_WIDTH / 4, DESIGN_HEIGHT)).toBeCloseTo(0.25, 5);
    });

    it("uses the smaller of the two ratios (height-limited)", () => {
        expect(computeScale(DESIGN_WIDTH / 2, DESIGN_HEIGHT / 4)).toBeCloseTo(0.25, 5);
    });

    it("guards against zero/negative viewport values", () => {
        expect(computeScale(0, 0)).toBe(1);
        expect(computeScale(-100, -100)).toBe(1);
    });
});

describe("computeScale — reservedHeight (BottomSheet peek + safe-area)", () => {
    it("reservedHeight=0 is identical to the no-arg case (backward-compatible)", () => {
        // Exactly the design size → scale 1 with or without reservedHeight=0.
        expect(computeScale(DESIGN_WIDTH, DESIGN_HEIGHT, DESIGN_WIDTH, DESIGN_HEIGHT, 0)).toBe(1);
        // Arbitrary viewport — must equal the 4-arg call.
        expect(
            computeScale(DESIGN_WIDTH * 2, DESIGN_HEIGHT * 2, DESIGN_WIDTH, DESIGN_HEIGHT, 0),
        ).toBe(computeScale(DESIGN_WIDTH * 2, DESIGN_HEIGHT * 2));
    });

    it("reservedHeight consumes extra space — 100px reserve on a viewport 100px taller than design → scale 1", () => {
        // viewport is design + 100px tall, but we reserve 100px → effective height = design height → scale 1
        expect(
            computeScale(DESIGN_WIDTH, DESIGN_HEIGHT + 100, DESIGN_WIDTH, DESIGN_HEIGHT, 100),
        ).toBe(1);
    });

    it("when reservedHeight >= viewportHeight, effective height is 0 → guard returns 1 (no negative/NaN scale)", () => {
        expect(computeScale(DESIGN_WIDTH, 32, DESIGN_WIDTH, DESIGN_HEIGHT, 32)).toBe(1);
        expect(computeScale(DESIGN_WIDTH, 10, DESIGN_WIDTH, DESIGN_HEIGHT, 100)).toBe(1);
    });

    it("reservedHeight makes height the limiting ratio (width-generous viewport, tall reserve → height-limited result)", () => {
        // Wide viewport (10× design width, never width-limited), but effective height = design/2 → scale 0.5
        const result = computeScale(DESIGN_WIDTH * 10, DESIGN_HEIGHT, DESIGN_WIDTH, DESIGN_HEIGHT, DESIGN_HEIGHT / 2);
        expect(result).toBeCloseTo(0.5, 5);
    });
});
