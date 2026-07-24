import { beforeEach, describe, expect, it, vi } from "vitest";
import type { EntrySummary, Toast } from "$lib/bridge/types";
import { entryDeletion } from "./entryDeletion.svelte";
import { selection } from "./selection.svelte";
import { vault } from "./vault.svelte";

const mocks = vi.hoisted(() => ({
  entryDelete: vi.fn(),
  entryDeletePermanent: vi.fn(),
  entryRestore: vi.fn(),
  toastInfo: vi.fn(),
  toastSuccess: vi.fn(),
  toastError: vi.fn(),
}));

vi.mock("$lib/bridge/entries", () => ({
  entryDelete: mocks.entryDelete,
  entryDeletePermanent: mocks.entryDeletePermanent,
  entryRestore: mocks.entryRestore,
}));

vi.mock("$lib/stores/toast.svelte", () => ({
  toast: {
    info: mocks.toastInfo,
    success: mocks.toastSuccess,
    error: mocks.toastError,
  },
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
  it("moves an active entry immediately and offers undo", async () => {
    mocks.entryDelete.mockResolvedValue(undefined);

    await entryDeletion.request(entry.id, entry.title);

    expect(mocks.entryDelete).toHaveBeenCalledWith(entry.id);
    expect(entryDeletion.pending).toBeNull();
    expect(vault.entries).toEqual([{ ...entry, trashed: true }]);
    expect(selection.selectedId).toBeNull();
    expect(mocks.toastInfo).toHaveBeenCalledWith(
      'Moved "Recovery password" to Recycle Bin',
      8000,
      expect.objectContaining({ label: "Undo" }),
    );
  });

  it("restores an entry from the undo action", async () => {
    mocks.entryDelete.mockResolvedValue(undefined);
    mocks.entryRestore.mockResolvedValue(undefined);
    await entryDeletion.request(entry.id, entry.title);
    const action = mocks.toastInfo.mock.calls[0][2] as NonNullable<Toast["action"]>;

    await action.run();

    expect(mocks.entryRestore).toHaveBeenCalledWith(entry.id);
    expect(vault.entries).toEqual([{ ...entry, trashed: false }]);
    expect(mocks.toastSuccess).toHaveBeenCalledWith('Restored "Recovery password"');
  });

  it("requires confirmation before permanently deleting a trashed entry", async () => {
    mocks.entryDeletePermanent.mockResolvedValue(undefined);
    vault.setEntries([{ ...entry, trashed: true }]);

    await entryDeletion.request(entry.id, entry.title);

    expect(entryDeletion.pending?.permanent).toBe(true);
    expect(mocks.entryDeletePermanent).not.toHaveBeenCalled();
    await entryDeletion.confirm();
    expect(mocks.entryDeletePermanent).toHaveBeenCalledWith(entry.id);
    expect(vault.entries).toEqual([]);
  });

  it("keeps the entry when moving it to the bin fails", async () => {
    const error = new Error("Delete failed");
    mocks.entryDelete.mockRejectedValue(error);
    const consoleError = vi.spyOn(console, "error").mockImplementation(() => {});

    await entryDeletion.request(entry.id, entry.title);

    expect(vault.entries).toEqual([entry]);
    expect(selection.selectedId).toBe(entry.id);
    expect(mocks.toastError).toHaveBeenCalledWith("Delete failed");
    consoleError.mockRestore();
  });
});
