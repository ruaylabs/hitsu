import { describe, expect, it } from "vitest";
import type { EntrySummary } from "$lib/bridge/types";
import { entryHaystack } from "./search";

function summary(overrides: Partial<EntrySummary> = {}): EntrySummary {
  return {
    id: "1",
    type: "login",
    title: "GitHub",
    subtitle: "erick",
    tags: [],
    favorite: false,
    ...overrides,
  };
}

describe("entryHaystack", () => {
  it("matches lowercased title, subtitle, url, username and tags", () => {
    const e = summary({
      url: "https://GitHub.com",
      username: "Erick@Navarro.io",
      tags: ["Work", "Dev"],
    });

    for (const q of ["github", "erick", "github.com", "navarro", "work", "dev"]) {
      expect(entryHaystack(e)).toContain(q);
    }
  });

  it("does not match across field boundaries", () => {
    const e = summary({ title: "abc", subtitle: "def" });
    expect(entryHaystack(e)).not.toContain("abcdef");
  });

  it("returns the cached haystack for the same summary object", () => {
    const e = summary();
    expect(entryHaystack(e)).toBe(entryHaystack(e));
  });
});
