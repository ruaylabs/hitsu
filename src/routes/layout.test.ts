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
  vi.restoreAllMocks();
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

  it("uses a safe fallback for non-message rejection values", async () => {
    render(Layout);

    window.dispatchEvent(rejectionEvent({ privateContext: true }));
    await tick();

    expect(screen.getByText("An unexpected background error occurred")).toBeInTheDocument();
  });
});
