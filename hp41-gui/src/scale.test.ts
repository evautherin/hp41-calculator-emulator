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
