import { type IconMenuItemOptions, Menu } from "@tauri-apps/api/menu";
import { fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { beforeEach, describe, expect, it, vi } from "vitest";
import * as entriesBridge from "$lib/bridge/entries";
import type { Entry, EntrySummary } from "$lib/bridge/types";
import { clipboard } from "$lib/stores/clipboard.svelte";
import { entryDeletion } from "$lib/stores/entryDeletion.svelte";
import { health } from "$lib/stores/health.svelte";
import { recycleBin } from "$lib/stores/recycleBin.svelte";
import { selection } from "$lib/stores/selection.svelte";
import { vault } from "$lib/stores/vault.svelte";
import { openHttpUrl } from "$lib/utils/openHttpUrl";
import ItemList from "./ItemList.svelte";

vi.mock("$lib/bridge/entries", async (importOriginal) => ({
  ...(await importOriginal<typeof import("$lib/bridge/entries")>()),
  entriesSearch: vi.fn(),
  entriesHealthReport: vi.fn(),
  entryDelete: vi.fn(),
  entryRestore: vi.fn(),
  entryUpdate: vi.fn(),
}));

vi.mock("$lib/stores/clipboard.svelte", () => ({
  clipboard: {
    copyPlain: vi.fn(),
    copySecretField: vi.fn(),
  },
}));

vi.mock("$lib/utils/openHttpUrl", () => ({ openHttpUrl: vi.fn() }));
vi.mock("@tauri-apps/api/menu", async (importOriginal) => ({
  ...(await importOriginal<typeof import("@tauri-apps/api/menu")>()),
  Menu: { new: vi.fn() },
}));

const menuNewMock = vi.mocked(Menu.new);
const menuPopupMock = vi.fn();
const menuCloseMock = vi.fn();

async function openRowMenu(row: HTMLElement) {
  menuPopupMock.mockClear();
  await fireEvent.contextMenu(row, { clientX: 50, clientY: 60 });
  await waitFor(() => expect(menuPopupMock).toHaveBeenCalledOnce());
}

function nativeItem(text: string): IconMenuItemOptions {
  const items = menuNewMock.mock.calls.at(-1)?.[0]?.items ?? [];
  const item = items.find((item) => "text" in item && item.text === text);
  if (!item) throw new Error(`Missing native menu item: ${text}`);
  return item as IconMenuItemOptions;
}

function runNativeItem(text: string) {
  const item = nativeItem(text);
  expect(item.enabled).not.toBe(false);
  item.action?.(item.id ?? "");
}

const entriesSearchMock = vi.mocked(entriesBridge.entriesSearch);
const entriesHealthReportMock = vi.mocked(entriesBridge.entriesHealthReport);
const entryDeleteMock = vi.mocked(entriesBridge.entryDelete);
const entryRestoreMock = vi.mocked(entriesBridge.entryRestore);
const entryUpdateMock = vi.mocked(entriesBridge.entryUpdate);
const copyPlainMock = vi.mocked(clipboard.copyPlain);
const copySecretFieldMock = vi.mocked(clipboard.copySecretField);
const openHttpUrlMock = vi.mocked(openHttpUrl);

function listRows(): HTMLElement[] {
  return Array.from(screen.getByRole("listbox").querySelectorAll<HTMLElement>("[role='option']"));
}

function makeEntries(count: number): EntrySummary[] {
  return Array.from({ length: count }, (_, i) => ({
    id: `id-${i}`,
    type: "login" as const,
    title: `Entry ${i}`,
    subtitle: `user${i}@example.com`,
    username: `user${i}@example.com`,
    url: "https://example.com",
    hasPassword: true,
    hasTotp: true,
    modifiedAt: new Date(Date.UTC(2025, 0, i + 1)).toISOString(),
    tags: i % 2 === 0 ? ["even"] : [],
    favorite: false,
  }));
}

function makeEntry(summary: EntrySummary): Entry {
  return {
    id: summary.id,
    type: summary.type,
    title: summary.title,
    subtitle: summary.subtitle,
    url: summary.url,
    username: summary.username,
    hasPassword: summary.hasPassword ?? false,
    hasTotp: summary.hasTotp ?? false,
    tags: summary.tags,
    favorite: summary.favorite,
    trashed: summary.trashed,
    folderId: summary.folderId,
    iconHint: summary.iconHint,
    attachments: [],
    customFields: [],
    modifiedAt: summary.modifiedAt ?? "",
    createdAt: "",
    historyCount: 0,
    hasCustomIcon: summary.hasCustomIcon ?? false,
  };
}

beforeEach(() => {
  menuPopupMock.mockReset().mockResolvedValue(undefined);
  menuCloseMock.mockReset().mockResolvedValue(undefined);
  menuNewMock.mockReset().mockResolvedValue({
    popup: menuPopupMock,
    close: menuCloseMock,
  } as unknown as Menu);
  localStorage.clear();
  entriesSearchMock.mockReset();
  entriesSearchMock.mockRejectedValue(new Error("backend search unavailable"));
  entriesHealthReportMock.mockReset();
  entriesHealthReportMock.mockResolvedValue({
    weak: [],
    reused: [],
  });
  health.reset();
  entryDeleteMock.mockReset();
  entryDeleteMock.mockResolvedValue(undefined);
  entryRestoreMock.mockReset();
  entryRestoreMock.mockResolvedValue(undefined);
  entryUpdateMock.mockReset();
  entryUpdateMock.mockResolvedValue(undefined as never);
  copyPlainMock.mockReset();
  copySecretFieldMock.mockReset();
  openHttpUrlMock.mockReset();
  entryDeletion.cancel();
  recycleBin.cancel();
  vault.setEditingId(null);
  selection.selectedId = null;
  selection.search = "";
  selection.filter = { kind: "all" };
});

describe("ItemList", () => {
  it("renders only a window of rows for large lists", () => {
    vault.setEntries(makeEntries(500));
    render(ItemList);

    const rows = listRows();
    expect(rows.length).toBeGreaterThan(0);
    // jsdom reports a zero-height viewport, so only the overscan rows
    // exist; the point is that nowhere near all 500 are in the DOM.
    expect(rows.length).toBeLessThan(50);
    expect(rows[0]).toHaveTextContent("Entry 0");
  });

  it("filters entries by search across fields", async () => {
    vault.setEntries(makeEntries(20));
    entriesSearchMock.mockResolvedValue({ ids: [], truncated: false });
    render(ItemList);

    selection.search = "user13@";
    await waitFor(() => expect(entriesSearchMock).toHaveBeenCalledWith("user13@"));
    const match = await screen.findByRole("option", { name: /Entry 13/ });
    expect(match).toBeInTheDocument();
    expect(listRows()).toHaveLength(1);
  });

  it("keeps backend-only results and selection visible during summary refreshes", async () => {
    const entries = makeEntries(3);
    vault.setEntries(entries);
    entriesSearchMock.mockResolvedValue({ ids: ["id-0", "id-1"], truncated: false });
    render(ItemList);
    selection.search = "buried note";
    await waitFor(() => expect(listRows()).toHaveLength(2));
    selection.select("id-1", "keyboard");

    let resolveRefresh!: (result: { ids: string[]; truncated: boolean }) => void;
    entriesSearchMock.mockImplementationOnce(
      () =>
        new Promise((resolve) => {
          resolveRefresh = resolve;
        }),
    );
    const calls = entriesSearchMock.mock.calls.length;
    vault.setEntries(entries.map((entry) => ({ ...entry })));
    await waitFor(() => expect(entriesSearchMock).toHaveBeenCalledTimes(calls + 1));

    expect(listRows()).toHaveLength(2);
    expect(selection.selectedId).toBe("id-1");
    resolveRefresh({ ids: ["id-0", "id-1"], truncated: false });
    await waitFor(() => expect(listRows()).toHaveLength(2));
    expect(selection.selectedId).toBe("id-1");

    selection.search = "different query";
    await waitFor(() => expect(listRows()).toHaveLength(0));
  });

  it("uses backend matches for fields absent from entry summaries", async () => {
    vault.setEntries(makeEntries(20));
    entriesSearchMock.mockResolvedValue({ ids: ["id-7"], truncated: false });
    render(ItemList);

    selection.search = "buried note";

    const match = await screen.findByRole("option", { name: /Entry 7/ });
    expect(match).toBeInTheDocument();
    expect(listRows()).toHaveLength(1);
  });

  it("explains the entry-detail match limit without hiding summary matches", async () => {
    vault.setEntries(makeEntries(20));
    entriesSearchMock.mockResolvedValue({ ids: [], truncated: true });
    render(ItemList);

    await fireEvent.input(screen.getByRole("textbox"), {
      target: { value: "Entry 7" },
    });

    expect(
      await screen.findByText(
        "Matches from notes and other entry details are limited to 500. Keep typing to narrow your search.",
      ),
    ).toBeInTheDocument();
    expect(screen.getByRole("option", { name: /Entry 7/ })).toBeInTheDocument();

    await fireEvent.input(screen.getByRole("textbox"), { target: { value: "" } });
    expect(
      screen.queryByText(/Matches from notes and other entry details/),
    ).not.toBeInTheDocument();
  });

  it("filters a folder recursively", async () => {
    vault.setFolders([
      { id: "work", name: "Work" },
      { id: "clients", name: "Clients", parentId: "work" },
    ]);
    vault.setEntries([
      { ...makeEntries(1)[0], id: "work-entry", folderId: "work" },
      { ...makeEntries(1)[0], id: "client-entry", title: "Client", folderId: "clients" },
      { ...makeEntries(1)[0], id: "root-entry", title: "Root" },
    ]);
    selection.filter = { kind: "folder", folderId: "work" };
    render(ItemList);

    expect(await screen.findByRole("option", { name: /Entry 0/ })).toBeInTheDocument();
    expect(screen.getByRole("option", { name: /Client/ })).toBeInTheDocument();
    expect(screen.queryByRole("option", { name: /Root/ })).not.toBeInTheDocument();
  });

  it("filters entries by backend-computed health issue", async () => {
    vault.setEntries(makeEntries(3));
    entriesHealthReportMock.mockResolvedValue({
      weak: ["id-1"],
      reused: [],
    });
    await health.refresh();
    selection.filter = { kind: "health", issue: "weak" };
    render(ItemList);

    expect(await screen.findByRole("option", { name: /Entry 1/ })).toBeInTheDocument();
    expect(listRows()).toHaveLength(1);
  });

  it("shows a healthy empty state for health filters", async () => {
    vault.setEntries(makeEntries(3));
    selection.filter = { kind: "health", issue: "reused" };
    render(ItemList);

    expect(await screen.findByText("No reused passwords")).toBeInTheDocument();
  });

  it("clears the selection when the current category has no entries", async () => {
    vault.setEntries(makeEntries(2));
    selection.selectedId = "id-0";
    render(ItemList);

    selection.filter = { kind: "favorites" };

    await waitFor(() => expect(selection.selectedId).toBeNull());
    expect(screen.getByText("No entries in this view")).toBeInTheDocument();
  });

  it("shows the empty state when nothing matches", async () => {
    vault.setEntries(makeEntries(5));
    render(ItemList, { onCreate: vi.fn() });

    selection.search = "no such entry";
    expect(await screen.findByText('No items match "no such entry"')).toBeInTheDocument();
    expect(listRows()).toHaveLength(0);
  });

  it("offers to create the first entry", async () => {
    const onCreate = vi.fn();
    vault.setEntries([]);
    render(ItemList, { onCreate });

    await fireEvent.click(screen.getByRole("button", { name: "Create your first entry" }));
    expect(onCreate).toHaveBeenCalledOnce();
  });

  it("offers to search all items when the current filter hides a match", async () => {
    vault.setEntries(makeEntries(3));
    selection.filter = { kind: "favorites" };
    selection.search = "Entry 1";
    render(ItemList, { onCreate: vi.fn() });

    await fireEvent.click(await screen.findByRole("button", { name: "Search all items" }));
    expect(selection.filter).toEqual({ kind: "all" });
    expect(await screen.findByRole("option", { name: /Entry 1/ })).toBeInTheDocument();
  });

  it("restores selection and scroll position after clearing search", async () => {
    vault.setEntries(makeEntries(20));
    selection.selectedId = "id-3";
    render(ItemList);
    const list = screen.getByRole("listbox");
    list.scrollTop = 120;
    await fireEvent.scroll(list);

    await fireEvent.input(screen.getByRole("textbox"), { target: { value: "Entry 1" } });
    await waitFor(() => expect(selection.search).toBe("Entry 1"));
    await waitFor(() => expect(selection.selectedId).toBe("id-1"));

    await fireEvent.click(screen.getByRole("button", { name: "Clear search" }));

    expect(selection.selectedId).toBe("id-3");
    await waitFor(() => expect(list.scrollTop).toBe(120));
  });

  it("sorts entries by title or modification date", async () => {
    const entries = makeEntries(3);
    entries[0] = { ...entries[0], title: "Zulu", modifiedAt: "2025-01-02T00:00:00Z" };
    entries[1] = { ...entries[1], title: "Alpha", modifiedAt: "2025-01-03T00:00:00Z" };
    entries[2] = { ...entries[2], title: "Mike", modifiedAt: "2025-01-01T00:00:00Z" };
    vault.setEntries(entries);
    render(ItemList);
    const sort = screen.getByLabelText("Sort entries");

    await fireEvent.change(sort, { target: { value: "title" } });
    expect(listRows()[0]).toHaveTextContent("Alpha");

    await fireEvent.change(sort, { target: { value: "modified" } });
    expect(listRows()[0]).toHaveTextContent("Alpha");
    expect(listRows()[1]).toHaveTextContent("Zulu");
  });

  it("shows the 20 most recently modified entries in the Recent view", async () => {
    vault.setEntries(makeEntries(25));
    selection.filter = { kind: "recent" };
    render(ItemList);

    expect(await screen.findByRole("option", { name: /Entry 24/ })).toBeInTheDocument();
    expect(screen.queryByRole("option", { name: /Entry 0/ })).not.toBeInTheDocument();
  });

  it("requests the shared empty-bin confirmation from the trash header", async () => {
    vault.setEntries([
      ...makeEntries(2),
      { ...makeEntries(1)[0], id: "trashed", title: "Deleted entry", trashed: true },
    ]);
    selection.filter = { kind: "trash" };
    render(ItemList);
    await screen.findByRole("option", { name: /Deleted entry/ });

    await fireEvent.click(screen.getByRole("button", { name: "Empty…" }));

    expect(recycleBin.pending).toBe(true);
  });

  it("copies the selected username and password with keyboard shortcuts", async () => {
    vault.setEntries(makeEntries(2));
    render(ItemList);
    await waitFor(() => expect(selection.selectedId).toBe("id-0"));

    await fireEvent.keyDown(window, { key: "c", metaKey: true });
    expect(copyPlainMock).toHaveBeenCalledWith("user0@example.com");

    await fireEvent.keyDown(window, { key: "c", metaKey: true, shiftKey: true });
    expect(copySecretFieldMock).toHaveBeenCalledWith("id-0", "password");
  });

  it("lets native Copy handle highlighted detail text", async () => {
    vault.setEntries(makeEntries(2));
    render(ItemList);
    await waitFor(() => expect(selection.selectedId).toBe("id-0"));
    const note = document.createElement("p");
    note.textContent = "Selected part of a note";
    document.body.append(note);
    const range = document.createRange();
    range.selectNodeContents(note);
    window.getSelection()?.addRange(range);

    try {
      const event = new KeyboardEvent("keydown", { key: "c", ctrlKey: true, cancelable: true });
      window.dispatchEvent(event);
      expect(event.defaultPrevented).toBe(false);
      expect(copyPlainMock).not.toHaveBeenCalled();
    } finally {
      window.getSelection()?.removeAllRanges();
      note.remove();
    }
  });

  it.each(["MacIntel", "Linux x86_64", "Win32"])(
    "uses text-only context menus (%s)",
    async (platform) => {
      const platformSpy = vi.spyOn(navigator, "platform", "get").mockReturnValue(platform);
      try {
        vault.setEntries(makeEntries(1));
        render(ItemList);
        await openRowMenu(listRows()[0]);

        expect(nativeItem("Open URL").icon).toBeUndefined();
        expect(nativeItem("Add to favorites").icon).toBeUndefined();
        expect(nativeItem("Delete").icon).toBeUndefined();
        expect(nativeItem("Copy username").icon).toBeUndefined();
        expect(nativeItem("Copy password").icon).toBeUndefined();
        expect(nativeItem("Copy TOTP").icon).toBeUndefined();
        expect(nativeItem("Edit").icon).toBeUndefined();
      } finally {
        platformSpy.mockRestore();
      }
    },
  );

  it("provides row actions in the context menu", async () => {
    vault.setEntries(makeEntries(2));
    render(ItemList);
    const row = await screen.findByRole("option", { name: /Entry 0/ });

    await openRowMenu(row);
    expect(screen.queryByRole("menu")).not.toBeInTheDocument();
    expect(menuPopupMock).toHaveBeenCalledWith(expect.objectContaining({ x: 50, y: 60 }));
    runNativeItem("Copy username");
    expect(copyPlainMock).toHaveBeenCalledWith("user0@example.com");
    runNativeItem("Copy password");
    expect(copySecretFieldMock).toHaveBeenCalledWith("id-0", "password");
    runNativeItem("Copy TOTP");
    expect(copySecretFieldMock).toHaveBeenCalledWith("id-0", "totp");

    await openRowMenu(row);
    runNativeItem("Open URL");
    expect(openHttpUrlMock).toHaveBeenCalledWith("https://example.com");

    await openRowMenu(row);
    runNativeItem("Edit");
    expect(vault.editingId).toBe("id-0");

    await openRowMenu(row);
    runNativeItem("Delete");
    await waitFor(() => expect(entryDeleteMock).toHaveBeenCalledWith("id-0"));
    expect(entryDeletion.pending).toBeNull();
  });

  it("toggles favorites from the context menu", async () => {
    const entries = makeEntries(2);
    vault.setEntries(entries);
    entryUpdateMock.mockResolvedValue(makeEntry({ ...entries[0], favorite: true }));
    render(ItemList);
    const row = await screen.findByRole("option", { name: /Entry 0/ });

    await openRowMenu(row);
    runNativeItem("Add to favorites");

    await waitFor(() => expect(entryUpdateMock).toHaveBeenCalledWith("id-0", { favorite: true }));
    expect(vault.entries[0].favorite).toBe(true);
  });

  it("disables unavailable native actions for trashed entries", async () => {
    vault.setEntries([
      {
        ...makeEntries(1)[0],
        trashed: true,
        favorite: true,
        username: undefined,
        url: undefined,
        hasPassword: false,
        hasTotp: false,
      },
    ]);
    selection.filter = { kind: "trash" };
    render(ItemList);
    await openRowMenu(await screen.findByRole("option", { name: /Entry 0/ }));

    for (const text of [
      "Copy username",
      "Copy password",
      "Copy TOTP",
      "Open URL",
      "Edit",
      "Remove from favorites",
    ]) {
      expect(nativeItem(text).enabled).toBe(false);
    }
    expect(nativeItem("Delete").enabled).not.toBe(false);
  });

  it("respects the navigation guard before opening a native menu", async () => {
    vault.setEntries(makeEntries(2));
    render(ItemList);
    const row = await screen.findByRole("option", { name: /Entry 1/ });
    const clearGuard = selection.setNavigationGuard(() => false);
    try {
      await fireEvent.contextMenu(row);
      expect(menuNewMock).not.toHaveBeenCalled();
      expect(selection.selectedId).toBe("id-0");
    } finally {
      clearGuard();
    }
  });

  it("releases native menus on replacement and unmount and ignores stale actions", async () => {
    vault.setEntries(makeEntries(2));
    const { unmount } = render(ItemList);
    const row = await screen.findByRole("option", { name: /Entry 0/ });
    await openRowMenu(row);
    const staleEdit = nativeItem("Edit");
    await openRowMenu(row);
    expect(menuCloseMock).toHaveBeenCalledOnce();
    staleEdit.action?.("edit");
    expect(vault.editingId).toBeNull();
    const currentEdit = nativeItem("Edit");
    unmount();
    expect(menuCloseMock).toHaveBeenCalledTimes(2);
    currentEdit.action?.("edit");
    expect(vault.editingId).toBeNull();
  });

  it("closes a menu that finishes creating after unmount", async () => {
    let resolveMenu!: (menu: Menu) => void;
    menuNewMock.mockImplementationOnce(
      () =>
        new Promise((resolve) => {
          resolveMenu = resolve;
        }),
    );
    vault.setEntries(makeEntries(1));
    const { unmount } = render(ItemList);
    await fireEvent.contextMenu(await screen.findByRole("option", { name: /Entry 0/ }));
    await waitFor(() => expect(menuNewMock).toHaveBeenCalledOnce());
    unmount();
    resolveMenu({ popup: menuPopupMock, close: menuCloseMock } as unknown as Menu);
    await waitFor(() => expect(menuCloseMock).toHaveBeenCalledOnce());
    expect(menuPopupMock).not.toHaveBeenCalled();
  });

  it("persists the chosen sort mode", async () => {
    const entries = makeEntries(3);
    entries[0] = { ...entries[0], title: "Zulu" };
    entries[1] = { ...entries[1], title: "Alpha" };
    vault.setEntries(entries);
    render(ItemList);
    const sort = screen.getByLabelText("Sort entries");

    await fireEvent.change(sort, { target: { value: "title" } });
    expect(localStorage.getItem("hitsu:list-sort")).toBe("title");
    expect(listRows()[0]).toHaveTextContent("Alpha");
  });

  it("restores the persisted sort mode on mount", () => {
    localStorage.setItem("hitsu:list-sort", "title");
    const entries = makeEntries(3);
    entries[0] = { ...entries[0], title: "Zulu" };
    entries[1] = { ...entries[1], title: "Alpha" };
    vault.setEntries(entries);
    render(ItemList);

    expect(listRows()[0]).toHaveTextContent("Alpha");
  });

  it("falls back to vault order for unknown persisted values", () => {
    localStorage.setItem("hitsu:list-sort", "bogus");
    vault.setEntries(makeEntries(3));
    render(ItemList);

    expect(listRows()[0]).toHaveTextContent("Entry 0");
  });

  it("offers to create the filtered type when a type view is empty", async () => {
    const onCreate = vi.fn();
    vault.setEntries(makeEntries(1));
    selection.filter = { kind: "type", type: "card" };
    render(ItemList, { onCreate });

    expect(await screen.findByText("No cards in this view")).toBeInTheDocument();
    await fireEvent.click(screen.getByRole("button", { name: "Create a card entry" }));
    expect(onCreate).toHaveBeenCalledOnce();
  });

  it("navigates filtered results with arrows while keeping search focused", async () => {
    vault.setEntries(makeEntries(5));
    selection.search = "Entry";
    render(ItemList);
    const search = screen.getByRole("textbox", { name: "Search all items…" });
    search.focus();
    await waitFor(() => expect(listRows()).toHaveLength(5));
    await waitFor(() => expect(selection.selectedId).toBe("id-0"));

    await fireEvent.keyDown(search, { key: "ArrowDown" });
    expect(selection.selectedId).toBe("id-1");
    expect(search).toHaveFocus();
    await fireEvent.keyDown(search, { key: "ArrowUp" });
    expect(selection.selectedId).toBe("id-0");
    await fireEvent.keyDown(search, { key: "ArrowUp" });
    expect(selection.selectedId).toBe("id-0");

    for (let i = 0; i < 6; i++) {
      await fireEvent.keyDown(search, { key: "ArrowDown" });
    }
    expect(selection.selectedId).toBe("id-4");
    expect(search).toHaveFocus();

    await fireEvent.input(search, { target: { value: "Entry 2" } });
    await waitFor(() => expect(listRows()).toHaveLength(1));
    await fireEvent.keyDown(search, { key: "ArrowDown" });
    expect(selection.selectedId).toBe("id-2");
    expect(search).toHaveFocus();
  });

  it("preserves text-editing keys and composition in search", async () => {
    vault.setEntries(makeEntries(3));
    render(ItemList);
    const search = screen.getByRole("textbox", { name: "Search all items…" });
    search.focus();
    await waitFor(() => expect(selection.selectedId).toBe("id-0"));

    for (const event of [
      { key: "End" },
      { key: "Home" },
      { key: "ArrowDown", shiftKey: true },
      { key: "ArrowDown", metaKey: true },
      { key: "ArrowDown", ctrlKey: true },
      { key: "ArrowDown", altKey: true },
      { key: "ArrowDown", isComposing: true },
    ]) {
      expect(await fireEvent.keyDown(search, event)).toBe(true);
      expect(selection.selectedId).toBe("id-0");
    }
    expect(search).toHaveFocus();
  });

  it("exposes full filtered positions even when earlier results are virtualized", async () => {
    vault.setEntries(makeEntries(40));
    selection.filter = { kind: "tag", tag: "even" };
    render(ItemList);
    const list = screen.getByRole("listbox", { name: "Entries" });
    await waitFor(() => expect(listRows()[0]).toHaveAttribute("aria-posinset", "1"));
    expect(listRows()[0]).toHaveAttribute("aria-setsize", "20");

    list.scrollTop = 580;
    await fireEvent.scroll(list);
    await waitFor(() => expect(listRows()[0]).toHaveAttribute("data-entry-id", "id-10"));
    expect(listRows()[0]).toHaveAttribute("aria-posinset", "6");
    expect(listRows()[0]).toHaveAttribute("aria-setsize", "20");

    // Tab can still enter the list when the selected option was scrolled away.
    expect(list).toHaveAttribute("tabindex", "0");
    list.focus();
    await waitFor(() =>
      expect(screen.getByRole("option", { name: /Entry/, selected: true })).toHaveFocus(),
    );
    expect(list).toHaveAttribute("tabindex", "-1");
  });

  it("announces accepted search navigation without moving focus or exposing subtitles", async () => {
    vault.setEntries(makeEntries(3));
    render(ItemList);
    const search = screen.getByRole("textbox");
    search.focus();
    await waitFor(() => expect(selection.selectedId).toBe("id-0"));
    const status = screen.getByRole("status");
    expect(status.textContent).toBe("");
    const releaseGuard = selection.setNavigationGuard(() => false);
    try {
      await fireEvent.keyDown(search, { key: "ArrowDown" });
      expect(status.textContent).toBe("");
      expect(selection.selectedId).toBe("id-0");
    } finally {
      releaseGuard();
    }
    await fireEvent.keyDown(search, { key: "ArrowDown" });
    expect(status).toHaveTextContent(/^Entry 1, 2 of 3$/);
    expect(search).toHaveFocus();
    expect(screen.getByRole("option", { name: /Entry/, selected: true })).toHaveAttribute(
      "data-entry-id",
      "id-1",
    );
    await fireEvent.input(search, { target: { value: "Entry 2" } });
    await waitFor(() => expect(status.textContent).toBe(""));
  });

  it("focuses and describes the selected option after jumping past the rendered window", async () => {
    vault.setEntries(makeEntries(40));
    render(ItemList);
    await waitFor(() => expect(selection.selectedId).toBe("id-0"));
    await fireEvent.keyDown(window, { key: "End" });
    const last = await screen.findByRole("option", { name: /Entry/, selected: true });
    await waitFor(() => expect(last).toHaveFocus());
    expect(last).toHaveAttribute("aria-posinset", "40");
    expect(last).toHaveAttribute("aria-setsize", "40");
    expect(screen.getByRole("status").textContent).toBe("");
    await fireEvent.keyDown(last, { key: "Home" });
    await waitFor(() =>
      expect(screen.getByRole("option", { name: /Entry/, selected: true })).toHaveFocus(),
    );
    expect(selection.selectedId).toBe("id-0");
  });

  it("moves selection with arrow keys", async () => {
    vault.setEntries(makeEntries(5));
    render(ItemList);

    // First entry is auto-selected.
    await waitFor(() => {
      expect(
        listRows().find((row) => row.getAttribute("aria-selected") === "true"),
      ).toHaveTextContent("Entry 0");
    });

    await fireEvent.keyDown(window, { key: "ArrowDown" });
    expect(selection.selectedId).toBe("id-1");

    await fireEvent.keyDown(window, { key: "End" });
    expect(selection.selectedId).toBe("id-4");
  });
});
