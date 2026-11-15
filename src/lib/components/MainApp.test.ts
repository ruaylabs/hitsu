import { cleanup, render, waitFor } from "@testing-library/svelte";
import { tick } from "svelte";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { app } from "$lib/stores/app.svelte";
import { selection } from "$lib/stores/selection.svelte";
import MainApp from "./MainApp.svelte";

const mocks = vi.hoisted(() => ({
  listeners: new Map<string, () => void>(),
  unlisten: vi.fn(),
  lock: vi.fn(),
  newItem: vi.fn(),
  shortcuts: vi.fn(),
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(async (name: string, handler: () => void) => {
    mocks.listeners.set(name, handler);
    return mocks.unlisten;
  }),
}));
vi.mock("$lib/bridge/prefs", () => ({ prefsGet: vi.fn(async () => ({})) }));
vi.mock("$lib/stores/vault.svelte", () => ({
  vault: { locked: true, meta: null, lock: mocks.lock },
}));
vi.mock("$lib/components/chrome/StatusBar.svelte", () => ({ default: () => {} }));
vi.mock("$lib/components/detail/ItemDetail.svelte", () => ({ default: () => {} }));
vi.mock("$lib/components/list/ItemList.svelte", () => ({ default: () => {} }));
vi.mock("$lib/components/settings/SettingsView.svelte", () => ({ default: () => {} }));
vi.mock("$lib/components/sidebar/Sidebar.svelte", () => ({ default: () => {} }));
vi.mock("$lib/components/ui/CommandPalette.svelte", () => ({ default: mocks.newItem }));
vi.mock("$lib/components/ui/ShortcutsDialog.svelte", () => ({ default: mocks.shortcuts }));

beforeEach(() => {
  vi.clearAllMocks();
  mocks.listeners.clear();
  app.view = "main";
});
afterEach(cleanup);

describe("native menu actions", () => {
  it("opens the new-item picker and keyboard shortcuts", async () => {
    render(MainApp);
    mocks.listeners.get("menu://new-item")?.();
    await tick();
    expect(mocks.newItem).toHaveBeenCalled();
    mocks.listeners.get("menu://keyboard-shortcuts")?.();
    await tick();
    expect(mocks.shortcuts).toHaveBeenCalled();
  });

  it("runs Lock Vault through the unsaved-edit guard", () => {
    render(MainApp);
    let pending: (() => void) | undefined;
    const removeGuard = selection.setNavigationGuard((navigate) => {
      pending = navigate;
      return false;
    });
    try {
      mocks.listeners.get("menu://lock-vault")?.();
      expect(mocks.lock).not.toHaveBeenCalled();
      expect(pending).toBeTypeOf("function");
      pending?.();
      expect(mocks.lock).toHaveBeenCalledOnce();
    } finally {
      removeGuard();
    }
  });

  it("returns from Settings and focuses search for Find", async () => {
    render(MainApp);
    app.view = "settings";
    const input = document.createElement("input");
    input.className = "entry-search-input";
    document.body.append(input);
    try {
      mocks.listeners.get("menu://find")?.();
      await waitFor(() => expect(input).toHaveFocus());
      expect(app.view).toBe("main");
    } finally {
      input.remove();
    }
  });

  it("unsubscribes all native menu and vault listeners on unmount", async () => {
    const { unmount } = render(MainApp);
    const listenerCount = mocks.listeners.size;
    expect(listenerCount).toBe(7);
    unmount();
    await waitFor(() => expect(mocks.unlisten).toHaveBeenCalledTimes(listenerCount));
  });
});
