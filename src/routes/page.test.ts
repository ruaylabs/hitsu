import { render, screen } from "@testing-library/svelte";
import { tick } from "svelte";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import Page from "./+page.svelte";

// The page kicks off security.load() at module-evaluation time (to overlap
// the IPC roundtrip with mounting), so the mock must already resolve when
// +page.svelte is imported — not just from beforeEach.
const mocks = vi.hoisted(() => ({
  loadSecurity: vi.fn().mockResolvedValue({}),
}));

vi.mock("$lib/stores/security.svelte", () => ({
  security: {
    load: mocks.loadSecurity,
  },
}));

beforeEach(() => {
  mocks.loadSecurity.mockResolvedValue({});
  vi.spyOn(document, "hasFocus").mockReturnValue(true);
});

afterEach(() => {
  vi.restoreAllMocks();
});

describe("privacy screen", () => {
  it("covers and disables the app while the window is unfocused", async () => {
    const { container } = render(Page);
    await tick();
    expect(screen.queryByRole("status", { name: "Privacy screen" })).not.toBeInTheDocument();

    window.dispatchEvent(new FocusEvent("blur"));
    await tick();

    expect(screen.getByRole("status", { name: "Privacy screen" })).toBeInTheDocument();
    const appContent = container.querySelector<HTMLElement>(".app-content");
    expect(appContent?.inert).toBe(true);
    expect(appContent).toHaveAttribute("aria-hidden", "true");

    window.dispatchEvent(new FocusEvent("focus"));
    await tick();

    expect(screen.queryByRole("status", { name: "Privacy screen" })).not.toBeInTheDocument();
    expect(appContent?.inert).toBe(false);
  });
});
