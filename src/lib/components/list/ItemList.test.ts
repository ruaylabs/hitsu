import { fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { beforeEach, describe, expect, it, vi } from "vitest";
import * as entriesBridge from "$lib/bridge/entries";
import type { EntrySummary } from "$lib/bridge/types";
import { selection } from "$lib/stores/selection.svelte";
import { vault } from "$lib/stores/vault.svelte";
import ItemList from "./ItemList.svelte";

vi.mock("$lib/bridge/entries", async (importOriginal) => ({
  ...(await importOriginal<typeof import("$lib/bridge/entries")>()),
  entriesSearch: vi.fn(),
}));

const entriesSearchMock = vi.mocked(entriesBridge.entriesSearch);

function makeEntries(count: number): EntrySummary[] {
  return Array.from({ length: count }, (_, i) => ({
    id: `id-${i}`,
    type: "login" as const,
    title: `Entry ${i}`,
    subtitle: `user${i}@example.com`,
    tags: i % 2 === 0 ? ["even"] : [],
    favorite: false,
  }));
}

beforeEach(() => {
  entriesSearchMock.mockReset();
  entriesSearchMock.mockRejectedValue(new Error("backend search unavailable"));
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
    render(ItemList);

    selection.search = "no such entry";
    expect(await screen.findByText('No items match "no such entry"')).toBeInTheDocument();
    expect(screen.queryAllByRole("option")).toHaveLength(0);
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
