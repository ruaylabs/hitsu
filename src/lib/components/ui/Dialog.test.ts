import { fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { beforeAll, describe, expect, it, vi } from "vitest";
import DialogHarness from "../../../test/fixtures/DialogHarness.svelte";

beforeAll(() => {
  // jsdom has no layout and otherwise reports every element as hidden.
  Object.defineProperty(HTMLElement.prototype, "offsetParent", {
    configurable: true,
    get() {
      return document.body;
    },
  });
});

describe("Dialog", () => {
  it("closes on Escape", async () => {
    const onclose = vi.fn();
    render(DialogHarness, { onclose });

    await fireEvent.keyDown(window, { key: "Escape" });

    expect(onclose).toHaveBeenCalledOnce();
  });

  it("confirms on Enter when configured", async () => {
    const onconfirm = vi.fn();
    render(DialogHarness, { onclose: vi.fn(), onconfirm });

    await fireEvent.keyDown(window, { key: "Enter" });

    expect(onconfirm).toHaveBeenCalledOnce();
  });

  it("closes only when the backdrop itself is clicked", async () => {
    const onclose = vi.fn();
    const { container } = render(DialogHarness, { onclose });
    const backdrop = container.querySelector<HTMLElement>(".dialog-overlay");

    if (!backdrop) throw new Error("dialog backdrop was not rendered");
    await fireEvent.click(screen.getByRole("dialog"));
    expect(onclose).not.toHaveBeenCalled();

    await fireEvent.click(backdrop);
    expect(onclose).toHaveBeenCalledOnce();
  });

  it("wraps Tab and Shift+Tab inside the dialog", async () => {
    render(DialogHarness, { onclose: vi.fn() });
    const close = screen.getByRole("button", { name: "Close" });
    const last = screen.getByRole("button", { name: "Last action" });

    last.focus();
    await fireEvent.keyDown(window, { key: "Tab" });
    expect(close).toHaveFocus();

    await fireEvent.keyDown(window, { key: "Tab", shiftKey: true });
    expect(last).toHaveFocus();
  });

  it("restores focus when unmounted", async () => {
    const outside = document.createElement("button");
    outside.textContent = "Outside";
    document.body.append(outside);
    outside.focus();

    const { unmount } = render(DialogHarness, { onclose: vi.fn() });
    await waitFor(() => expect(screen.getByRole("dialog")).toHaveFocus());

    unmount();
    expect(outside).toHaveFocus();
    outside.remove();
  });
});
