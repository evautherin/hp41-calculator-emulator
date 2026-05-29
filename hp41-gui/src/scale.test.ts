import { describe, it, expect } from "vitest";
import { computeScale, DESIGN_WIDTH, DESIGN_HEIGHT } from "./scale";

describe("computeScale", () => {
    it("returns 1 when the viewport is at least the design size", () => {
        expect(computeScale(DESIGN_WIDTH, DESIGN_HEIGHT)).toBe(1);
        expect(computeScale(2000, 2000)).toBe(1);
    });

    it("never scales above 1 (no upscaling)", () => {
        expect(computeScale(10000, 10000)).toBe(1);
    });

    it("scales down by the height-limiting dimension", () => {
        expect(computeScale(DESIGN_WIDTH, DESIGN_HEIGHT / 2)).toBeCloseTo(0.5, 5);
    });

    it("scales down by the width-limiting dimension", () => {
        expect(computeScale(DESIGN_WIDTH / 4, DESIGN_HEIGHT)).toBeCloseTo(0.25, 5);
    });

    it("uses the smaller of the two ratios", () => {
        expect(computeScale(DESIGN_WIDTH / 2, DESIGN_HEIGHT / 4)).toBeCloseTo(0.25, 5);
    });

    it("guards against zero/negative viewport values", () => {
        expect(computeScale(0, 0)).toBe(1);
        expect(computeScale(-100, -100)).toBe(1);
    });
});
