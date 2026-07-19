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

  it("catches common passwords despite case, leet swaps, and padding", () => {
    expect(estimateStrength("Password123!").level).toBeLessThanOrEqual(1);
    expect(estimateStrength("qwerty2024").level).toBeLessThanOrEqual(1);
    expect(estimateStrength("P4ssw0rd").level).toBe(0);
    expect(estimateStrength("!!Sunshine99!!").level).toBeLessThanOrEqual(1);
  });

  it("caps repeated and sequential runs at zero with an honest label", () => {
    expect(estimateStrength("aaaaaaaaaaaa").level).toBe(0);
    expect(estimateStrength("abcdefghijkl")).toMatchObject({ level: 0, label: "Very weak" });
    expect(estimateStrength("0123456789").level).toBe(0);
    expect(estimateStrength("iloveyou").label).toBe("Very weak");
  });

  it("does not flag strong passphrases that merely contain a common word", () => {
    expect(estimateStrength("Correct-Horse-Battery-Staple-42").level).toBe(4);
    expect(estimateStrength("sunshine-glacier-parrot-42x").level).toBe(4);
  });
});
