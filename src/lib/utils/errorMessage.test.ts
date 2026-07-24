import { describe, expect, it } from "vitest";
import { errorMessage } from "./errorMessage";

describe("errorMessage", () => {
  it("returns an Error's message", () => {
    expect(errorMessage(new Error("Vault unavailable"))).toBe("Vault unavailable");
  });

  it("stringifies non-Error values", () => {
    expect(errorMessage("Request failed")).toBe("Request failed");
    expect(errorMessage(404)).toBe("404");
  });
});
