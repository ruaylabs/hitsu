import { describe, expect, it } from "vitest";
import { tagColor } from "./tagColor";

describe("tagColor", () => {
  it("returns stable colors regardless of tag casing", () => {
    expect(tagColor("Work")).toBe(tagColor("work"));
    expect(tagColor(" work ")).toBe(tagColor("work"));
  });

  it("distributes tag names across the palette", () => {
    const colors = new Set(["work", "personal", "critical", "homelab", "finance"].map(tagColor));
    expect(colors.size).toBeGreaterThan(2);
  });
});
