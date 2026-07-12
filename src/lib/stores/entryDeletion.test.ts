import { beforeEach, describe, expect, it, vi } from "vitest";
import type { EntrySummary } from "$lib/bridge/types";
import { entryDeletion } from "./entryDeletion.svelte";
import { selection } from "./selection.svelte";
import { vault } from "./vault.svelte";

const mocks = vi.hoisted(() => ({
  entryDelete: vi.fn(),
  entryDeletePermanent: vi.fn(),
  toastError: vi.fn(),
}));

vi.mock("$lib/bridge/entries", () => ({
  entryDelete: mocks.entryDelete,
  entryDeletePermanent: mocks.entryDeletePermanent,
}));

vi.mock("$lib/stores/toast.svelte", () => ({
  toast: { error: mocks.toastError },
}));

const entry: EntrySummary = {
  id: "entry-1",
  type: "password",
  title: "Recovery password",
  subtitle: "",
  tags: [],
  favorite: false,
};

beforeEach(() => {
  vi.clearAllMocks();
  entryDeletion.cancel();
  vault.setEntries([entry]);
  selection.selectedId = entry.id;
});

describe("entry deletion", () => {
  it("moves an active entry to the recycle bin and clears selection", async () => {
    mocks.entryDelete.mockResolvedValue(undefined);
    entryDeletion.request(entry.id, entry.title);

    await entryDeletion.confirm();

    expect(mocks.entryDelete).toHaveBeenCalledWith(entry.id);
    expect(vault.entries).toEqual([{ ...entry, trashed: true }]);
    expect(selection.selectedId).toBeNull();
  });

  it("permanently removes an entry already in the recycle bin", async () => {
    mocks.entryDeletePermanent.mockResolvedValue(undefined);
    vault.setEntries([{ ...entry, trashed: true }]);
    entryDeletion.request(entry.id, entry.title);

    await entryDeletion.confirm();

    expect(mocks.entryDeletePermanent).toHaveBeenCalledWith(entry.id);
    expect(vault.entries).toEqual([]);
  });

  it("keeps the entry when deletion fails", async () => {
    const error = new Error("Delete failed");
    mocks.entryDelete.mockRejectedValue(error);
    const consoleError = vi.spyOn(console, "error").mockImplementation(() => {});
    entryDeletion.request(entry.id, entry.title);

    await entryDeletion.confirm();

    expect(vault.entries).toEqual([entry]);
    expect(selection.selectedId).toBe(entry.id);
    expect(mocks.toastError).toHaveBeenCalledWith("Delete failed");
    consoleError.mockRestore();
  });
});
