import { fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { beforeEach, describe, expect, it, vi } from "vitest";
import * as entriesBridge from "$lib/bridge/entries";
import type { EntrySummary } from "$lib/bridge/types";
import { clipboard } from "$lib/stores/clipboard.svelte";
import { entryDeletion } from "$lib/stores/entryDeletion.svelte";
import { selection } from "$lib/stores/selection.svelte";
import { vault } from "$lib/stores/vault.svelte";
import { openHttpUrl } from "$lib/utils/openHttpUrl";
import ItemList from "./ItemList.svelte";

vi.mock("$lib/bridge/entries", async (importOriginal) => ({
  ...(await importOriginal<typeof import("$lib/bridge/entries")>()),
  entriesSearch: vi.fn(),
}));

vi.mock("$lib/stores/clipboard.svelte", () => ({
  clipboard: {
    copyPlain: vi.fn(),
    copySecretField: vi.fn(),
  },
}));

vi.mock("$lib/utils/openHttpUrl", () => ({ openHttpUrl: vi.fn() }));

const entriesSearchMock = vi.mocked(entriesBridge.entriesSearch);
const copyPlainMock = vi.mocked(clipboard.copyPlain);
const copySecretFieldMock = vi.mocked(clipboard.copySecretField);
const openHttpUrlMock = vi.mocked(openHttpUrl);

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

beforeEach(() => {
  entriesSearchMock.mockReset();
  entriesSearchMock.mockRejectedValue(new Error("backend search unavailable"));
  copyPlainMock.mockReset();
  copySecretFieldMock.mockReset();
  openHttpUrlMock.mockReset();
  entryDeletion.cancel();
  vault.setEditingId(null);
  selection.selectedId = null;
  selection.search = "";
  selection.filter = { kind: "all" };
});

describe("ItemList", () => {
  it("renders only a window of rows for large lists", () => {
    vault.setEntries(makeEntries(500));
    render(ItemList);

    const rows = screen.getAllByRole("option");
    expect(rows.length).toBeGreaterThan(0);
    // jsdom reports a zero-height viewport, so only the overscan rows
    // exist; the point is that nowhere near all 500 are in the DOM.
    expect(rows.length).toBeLessThan(50);
    expect(rows[0]).toHaveTextContent("Entry 0");
  });

  it("filters entries by search across fields", async () => {
    vault.setEntries(makeEntries(20));
    entriesSearchMock.mockResolvedValue([]);
    render(ItemList);

    selection.search = "user13@";
    await waitFor(() => expect(entriesSearchMock).toHaveBeenCalledWith("user13@"));
    const match = await screen.findByRole("option", { name: /Entry 13/ });
    expect(match).toBeInTheDocument();
    expect(screen.getAllByRole("option")).toHaveLength(1);
  });

  it("uses backend matches for fields absent from entry summaries", async () => {
    vault.setEntries(makeEntries(20));
    entriesSearchMock.mockResolvedValue(["id-7"]);
    render(ItemList);

    selection.search = "buried note";

    const match = await screen.findByRole("option", { name: /Entry 7/ });
    expect(match).toBeInTheDocument();
    expect(screen.getAllByRole("option")).toHaveLength(1);
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

  it("shows the empty state when nothing matches", async () => {
    vault.setEntries(makeEntries(5));
    render(ItemList, { onCreate: vi.fn() });

    selection.search = "no such entry";
    expect(await screen.findByText('No items match "no such entry"')).toBeInTheDocument();
    expect(screen.queryAllByRole("option")).toHaveLength(0);
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

  it("shows the 20 most recently modified entries in the Recent view", async () => {
    vault.setEntries(makeEntries(25));
    selection.filter = { kind: "recent" };
    render(ItemList);

    expect(await screen.findByRole("option", { name: /Entry 24/ })).toBeInTheDocument();
    expect(screen.queryByRole("option", { name: /Entry 0/ })).not.toBeInTheDocument();
  });

  it("copies the selected username and password with keyboard shortcuts", async () => {
    vault.setEntries(makeEntries(2));
    render(ItemList);
    await screen.findByRole("option", { selected: true });

    await fireEvent.keyDown(window, { key: "c", metaKey: true });
    expect(copyPlainMock).toHaveBeenCalledWith("user0@example.com");

    await fireEvent.keyDown(window, { key: "c", metaKey: true, shiftKey: true });
    expect(copySecretFieldMock).toHaveBeenCalledWith("id-0", "password");
  });

  it("provides row actions in the context menu", async () => {
    vault.setEntries(makeEntries(2));
    render(ItemList);
    const row = await screen.findByRole("option", { name: /Entry 0/ });

    await fireEvent.contextMenu(row, { clientX: 50, clientY: 50 });
    expect(screen.getByRole("menu", { name: "Actions for Entry 0" })).toBeInTheDocument();
    await fireEvent.click(screen.getByRole("menuitem", { name: "Copy TOTP" }));
    expect(copySecretFieldMock).toHaveBeenCalledWith("id-0", "totp");

    await fireEvent.contextMenu(row, { clientX: 50, clientY: 50 });
    await fireEvent.click(screen.getByRole("menuitem", { name: "Open URL" }));
    expect(openHttpUrlMock).toHaveBeenCalledWith("https://example.com");

    await fireEvent.contextMenu(row, { clientX: 50, clientY: 50 });
    await fireEvent.click(screen.getByRole("menuitem", { name: "Edit" }));
    expect(vault.editingId).toBe("id-0");

    await fireEvent.contextMenu(row, { clientX: 50, clientY: 50 });
    await fireEvent.click(screen.getByRole("menuitem", { name: "Delete" }));
    expect(entryDeletion.pending?.id).toBe("id-0");
  });

  it("moves selection with arrow keys", async () => {
    vault.setEntries(makeEntries(5));
    render(ItemList);

    // First entry is auto-selected.
    expect(await screen.findByRole("option", { selected: true })).toHaveTextContent("Entry 0");

    await fireEvent.keyDown(window, { key: "ArrowDown" });
    expect(selection.selectedId).toBe("id-1");

    await fireEvent.keyDown(window, { key: "End" });
    expect(selection.selectedId).toBe("id-4");
  });
});
