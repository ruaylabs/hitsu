import { describe, expect, it } from "vitest";
import { isMacPlatform, keyboardShortcut } from "./keyboardShortcut";

describe("keyboard shortcuts", () => {
  it("formats macOS shortcuts with platform symbols", () => {
    expect(isMacPlatform("MacIntel")).toBe(true);
    expect(keyboardShortcut("K", { platform: "MacIntel" })).toBe("⌘K");
    expect(keyboardShortcut("C", { shift: true, platform: "MacIntel" })).toBe("⌘⇧C");
    expect(keyboardShortcut("Backspace", { platform: "MacIntel" })).toBe("⌘⌫");
  });

  it("formats Linux shortcuts with readable key names", () => {
    expect(isMacPlatform("Linux x86_64")).toBe(false);
    expect(keyboardShortcut("K", { platform: "Linux x86_64" })).toBe("Ctrl+K");
    expect(keyboardShortcut("C", { shift: true, platform: "Linux x86_64" })).toBe("Ctrl+Shift+C");
    expect(keyboardShortcut("Backspace", { platform: "Linux x86_64" })).toBe("Ctrl+Backspace");
  });
});
