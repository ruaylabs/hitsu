import { beforeEach, describe, expect, it, vi } from "vitest";
import { theme } from "./theme.svelte";

const mocks = vi.hoisted(() => ({ prefsSetTheme: vi.fn() }));

vi.mock("$lib/bridge/prefs", () => ({ prefsSetTheme: mocks.prefsSetTheme }));

beforeEach(() => {
  vi.clearAllMocks();
  document.documentElement.removeAttribute("data-theme");
  theme.hydrate("system");
});

describe("theme", () => {
  it("uses an explicit document theme for overrides", () => {
    theme.hydrate("dark");

    expect(theme.preference).toBe("dark");
    expect(document.documentElement).toHaveAttribute("data-theme", "dark");
  });

  it("removes the override when following the system", () => {
    theme.hydrate("light");
    theme.hydrate("system");

    expect(document.documentElement).not.toHaveAttribute("data-theme");
  });

  it("restores the previous theme when saving fails", async () => {
    mocks.prefsSetTheme.mockRejectedValue(new Error("Could not save theme"));
    theme.hydrate("light");

    await expect(theme.save("dark")).rejects.toThrow("Could not save theme");

    expect(theme.preference).toBe("light");
    expect(document.documentElement).toHaveAttribute("data-theme", "light");
  });
});
