import { describe, expect, it } from "vitest";
import { errorKind, errorMessage, isIpcErrorPayload, normalizeError } from "./errorMessage";

describe("errorMessage", () => {
  it("returns an Error's message", () => {
    expect(errorMessage(new Error("Vault unavailable"))).toBe("Vault unavailable");
  });

  it("reads structured IPC errors", () => {
    const error = { kind: "biometric_canceled", message: "Touch ID was canceled." };

    expect(isIpcErrorPayload(error)).toBe(true);
    expect(errorKind(error)).toBe("biometric_canceled");
    expect(errorMessage(error)).toBe("Touch ID was canceled.");
  });

  it("normalizes structured IPC errors without losing their kind", () => {
    const error = normalizeError({ kind: "biometric_failed", message: "Touch ID failed" });

    expect(error).toBeInstanceOf(Error);
    expect(error.message).toBe("Touch ID failed");
    expect(errorKind(error)).toBe("biometric_failed");
  });

  it("stringifies non-Error values", () => {
    expect(errorMessage("Request failed")).toBe("Request failed");
    expect(errorMessage(404)).toBe("404");
  });
});
