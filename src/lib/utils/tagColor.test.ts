import { describe, expect, it } from "vitest";
import { tagColor } from "./tagColor";

describe("tagColor", () => {
  it("returns stable colors regardless of tag casing", () => {
    expect(tagColor("Work")).toEqual(tagColor("work"));
    expect(tagColor(" work ")).toEqual(tagColor("work"));
  });

  it("returns paired fill and text tokens", () => {
    const colors = tagColor("work");
    expect(colors.fill).toMatch(/^var\(--tag-.+-fill\)$/);
    expect(colors.text).toBe(colors.fill.replace("-fill", "-text"));
  });

  it("distributes tag names across the palette", () => {
    const colors = new Set(
      ["work", "personal", "critical", "homelab", "finance"].map((tag) => tagColor(tag).fill),
    );
    expect(colors.size).toBeGreaterThan(2);
  });
});
