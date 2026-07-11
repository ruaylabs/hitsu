import { describe, expect, it } from "vitest";
import { estimateStrength, strengthColor } from "./passwordStrength";

describe("password strength", () => {
  it("treats empty and short passwords as too short", () => {
    expect(estimateStrength("")).toEqual({ level: 0, fraction: 0, label: "Too short" });
    expect(estimateStrength("Ab1!").level).toBe(0);
  });

  it("rewards length and character diversity", () => {
    expect(estimateStrength("Correct-Horse-Battery-Staple-42").level).toBe(4);
  });

  it("penalizes common passwords", () => {
    expect(estimateStrength("password").level).toBe(0);
    expect(strengthColor(0)).toBe("var(--danger)");
  });
});
