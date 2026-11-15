import { render, screen } from "@testing-library/svelte";
import { tick } from "svelte";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import Page from "./+page.svelte";

// The page kicks off security.load() at module-evaluation time (to overlap
// the IPC roundtrip with mounting), so the mock must already resolve when
// +page.svelte is imported — not just from beforeEach.
const mocks = vi.hoisted(() => ({
  loadSecurity: vi.fn().mockResolvedValue({}),
  hideOnBlur: true,
}));

vi.mock("$lib/stores/security.svelte", () => ({
  security: {
    load: mocks.loadSecurity,
    get hideOnBlur() {
      return mocks.hideOnBlur;
    },
  },
}));

beforeEach(() => {
  mocks.loadSecurity.mockResolvedValue({});
  mocks.hideOnBlur = true;
  vi.spyOn(document, "hasFocus").mockReturnValue(true);
});

afterEach(() => {
  vi.restoreAllMocks();
});

describe("inactive window state", () => {
  it("tracks focus independently of the hide-on-blur preference", async () => {
    mocks.hideOnBlur = false;
    render(Page);
    await tick();
    expect(document.documentElement).not.toHaveAttribute("data-window-inactive");

    window.dispatchEvent(new FocusEvent("blur"));
    await tick();
    expect(document.documentElement).toHaveAttribute("data-window-inactive");

    window.dispatchEvent(new FocusEvent("focus"));
    await tick();
    expect(document.documentElement).not.toHaveAttribute("data-window-inactive");
  });

  it("initializes from document focus and removes the attribute on unmount", async () => {
    vi.mocked(document.hasFocus).mockReturnValue(false);
    const { unmount } = render(Page);
    await tick();
    expect(document.documentElement).toHaveAttribute("data-window-inactive");

    unmount();
    expect(document.documentElement).not.toHaveAttribute("data-window-inactive");
  });
});

describe("privacy screen", () => {
  it("keeps content visible and enabled when hiding on blur is disabled", async () => {
    mocks.hideOnBlur = false;
    const { container } = render(Page);
    await tick();

    window.dispatchEvent(new FocusEvent("blur"));
    await tick();

    expect(screen.queryByRole("status", { name: "Privacy screen" })).not.toBeInTheDocument();
    const appContent = container.querySelector<HTMLElement>(".app-content");
    expect(appContent?.inert).toBe(false);
    expect(appContent).toHaveAttribute("aria-hidden", "false");
  });

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
