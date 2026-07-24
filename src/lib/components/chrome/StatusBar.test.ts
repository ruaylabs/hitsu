import { fireEvent, render, screen } from "@testing-library/svelte";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { clipboard } from "$lib/stores/clipboard.svelte";
import { vault } from "$lib/stores/vault.svelte";
import StatusBar from "./StatusBar.svelte";

const mocks = vi.hoisted(() => ({
  copyWithTimeout: vi.fn(),
  clear: vi.fn(),
}));

vi.mock("$lib/bridge/clipboard", () => ({
  clipboardCopy: vi.fn(),
  clipboardCopyWithTimeout: mocks.copyWithTimeout,
  clipboardClear: mocks.clear,
}));

beforeEach(() => {
  vi.useFakeTimers();
  vi.clearAllMocks();
  vault.setEntries([]);
  vault.setMeta(null);
  mocks.copyWithTimeout.mockResolvedValue(undefined);
  mocks.clear.mockResolvedValue(undefined);
});

afterEach(() => {
  clipboard.cancel();
  vi.useRealTimers();
});

describe("StatusBar", () => {
  it("clears a copied secret from the countdown button", async () => {
    await clipboard.copy("secret", 15);
    render(StatusBar, { onHelpClick: vi.fn(), onSettingsClick: vi.fn() });

    const clearButton = screen.getByRole("button", { name: "Clear clipboard now" });
    expect(clearButton).toHaveTextContent("Clears in 15s");
    await fireEvent.click(clearButton);

    expect(mocks.clear).toHaveBeenCalledOnce();
    expect(screen.queryByRole("button", { name: "Clear clipboard now" })).not.toBeInTheDocument();
  });
});
