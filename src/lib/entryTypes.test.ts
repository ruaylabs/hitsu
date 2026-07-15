import { describe, expect, it } from "vitest";
import { ENTRY_TYPE_BY_TYPE, ENTRY_TYPES } from "$lib/entryTypes";

const EXPECTED_TYPES = [
  "login",
  "password",
  "note",
  "identity",
  "card",
  "software_license",
  "passport",
];

describe("entry type metadata", () => {
  it("defines every entry type once and indexes each definition", () => {
    expect(ENTRY_TYPES.map((metadata) => metadata.type)).toEqual(EXPECTED_TYPES);
    expect(new Set(ENTRY_TYPES.map((metadata) => metadata.type)).size).toBe(ENTRY_TYPES.length);

    for (const metadata of ENTRY_TYPES) {
      expect(ENTRY_TYPE_BY_TYPE[metadata.type]).toBe(metadata);
    }
  });
});
