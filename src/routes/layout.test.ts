import { render, screen } from "@testing-library/svelte";
import { tick } from "svelte";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { toast } from "$lib/stores/toast.svelte";
import Layout from "./+layout.svelte";

function clearToasts() {
  for (const item of toast.all) toast.dismiss(item.id);
}

function rejectionEvent(reason: unknown) {
  const event = new Event("unhandledrejection", { cancelable: true }) as PromiseRejectionEvent;
  Object.defineProperty(event, "reason", { value: reason });
  return event;
}

beforeEach(() => {
  clearToasts();
  vi.spyOn(console, "error").mockImplementation(() => {});
});

afterEach(() => {
  clearToasts();
  delete document.documentElement.dataset.platform;
  vi.restoreAllMocks();
  vi.unstubAllEnvs();
});

describe("platform scrollbars", () => {
  it.each([
    ["MacIntel", "macos"],
    ["Linux x86_64", "linux"],
    ["Linux aarch64", "linux"],
    ["Win32", "other"],
    ["", "other"],
  ])("identifies %s as %s", (platform, expected) => {
    vi.spyOn(navigator, "platform", "get").mockReturnValue(platform);
    render(Layout);

    expect(document.documentElement.dataset.platform).toBe(expected);
  });
});

describe("webview context menus", () => {
  beforeEach(() => {
    vi.stubEnv("DEV", false);
  });

  function contextMenu(target: Element) {
    const event = new MouseEvent("contextmenu", { bubbles: true, cancelable: true });
    target.dispatchEvent(event);
    return event;
  }

  it("preserves the native menu on app chrome during development", () => {
    vi.stubEnv("DEV", true);
    render(Layout);
    expect(contextMenu(document.body).defaultPrevented).toBe(false);
  });

  it("suppresses the native menu on app chrome", () => {
    render(Layout);
    expect(contextMenu(document.body).defaultPrevented).toBe(true);
  });

  it.each(["input", "textarea"])("preserves the native menu in %s elements", (tag) => {
    render(Layout);
    const element = document.createElement(tag);
    document.body.append(element);
    expect(contextMenu(element).defaultPrevented).toBe(false);
    element.remove();
  });

  it.each(["text", "all"])("preserves menus for user-select: %s text and its children", (value) => {
    render(Layout);
    const element = document.createElement("div");
    element.style.userSelect = value;
    const child = element.appendChild(document.createElement("span"));
    document.body.append(element);
    expect(contextMenu(child).defaultPrevented).toBe(false);
    element.remove();
  });

  it("preserves editable content menus", () => {
    render(Layout);
    const element = document.createElement("div");
    Object.defineProperty(element, "isContentEditable", { value: true });
    document.body.append(element);
    expect(contextMenu(element).defaultPrevented).toBe(false);
    element.remove();
  });

  it("leaves custom context-menu handlers intact", () => {
    render(Layout);
    const element = document.createElement("div");
    const handler = vi.fn((event: Event) => event.preventDefault());
    element.addEventListener("contextmenu", handler);
    document.body.append(element);
    expect(contextMenu(element).defaultPrevented).toBe(true);
    expect(handler).toHaveBeenCalledOnce();
    element.remove();
  });

  it("removes the global handler when unmounted", () => {
    const { unmount } = render(Layout);
    unmount();
    expect(contextMenu(document.body).defaultPrevented).toBe(false);
  });
});

describe("toast actions", () => {
  it("runs an action and dismisses its toast", async () => {
    const action = vi.fn();
    toast.info("Moved to Recycle Bin", 0, { label: "Undo", run: action });
    render(Layout);

    await screen.getByRole("button", { name: "Undo" }).click();

    expect(action).toHaveBeenCalledOnce();
    expect(screen.queryByText("Moved to Recycle Bin")).not.toBeInTheDocument();
  });
});

describe("global rejection handling", () => {
  it("reports unhandled promise errors through the toast stack", async () => {
    render(Layout);
    const event = rejectionEvent(new Error("Background refresh failed"));

    window.dispatchEvent(event);
    await tick();

    expect(event.defaultPrevented).toBe(true);
    expect(screen.getByText("Background refresh failed")).toBeInTheDocument();
    expect(console.error).toHaveBeenCalledWith("Unhandled promise rejection", expect.any(Error));
  });

  it("reports structured IPC errors through the toast stack", async () => {
    render(Layout);

    window.dispatchEvent(
      rejectionEvent({ kind: "io", message: "The background file operation failed" }),
    );
    await tick();

    expect(screen.getByText("The background file operation failed")).toBeInTheDocument();
  });

  it("uses a safe fallback for non-message rejection values", async () => {
    render(Layout);

    window.dispatchEvent(rejectionEvent({ privateContext: true }));
    await tick();

    expect(screen.getByText("An unexpected background error occurred")).toBeInTheDocument();
  });
});
