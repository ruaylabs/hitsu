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

  it("uses backend matches for fields absent from entry summaries", async () => {
    vault.setEntries(makeEntries(20));
    entriesSearchMock.mockResolvedValue({ ids: ["id-7"], truncated: false });
    render(ItemList);

    selection.search = "buried note";

    const match = await screen.findByRole("option", { name: /Entry 7/ });
    expect(match).toBeInTheDocument();
    expect(listRows()).toHaveLength(1);
  });

  it("warns when the backend finds more than 500 full-field matches", async () => {
    vault.setEntries(makeEntries(20));
    entriesSearchMock.mockResolvedValue({ ids: [], truncated: true });
    render(ItemList);

    await fireEvent.input(screen.getByRole("textbox"), {
      target: { value: "buried note" },
    });

    expect(
      await screen.findByText(
        "More than 500 full-field matches were found; keep typing to narrow the search.",
      ),
    ).toBeInTheDocument();
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
    await waitFor(() => expect(entryDeleteMock).toHaveBeenCalledWith("id-0"));
    expect(entryDeletion.pending).toBeNull();
  });

  it("toggles favorites from the context menu", async () => {
    const entries = makeEntries(2);
    vault.setEntries(entries);
    entryUpdateMock.mockResolvedValue(makeEntry({ ...entries[0], favorite: true }));
    render(ItemList);
    const row = await screen.findByRole("option", { name: /Entry 0/ });

    await fireEvent.contextMenu(row, { clientX: 50, clientY: 50 });
    await fireEvent.click(screen.getByRole("menuitem", { name: "Add to favorites" }));

    await waitFor(() => expect(entryUpdateMock).toHaveBeenCalledWith("id-0", { favorite: true }));
    expect(vault.entries[0].favorite).toBe(true);
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
