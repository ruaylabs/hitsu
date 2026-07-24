import { beforeEach, describe, expect, it, vi } from "vitest";
import type { EntrySummary } from "$lib/bridge/types";
import { recycleBin } from "./recycleBin.svelte";
import { selection } from "./selection.svelte";
import { vault } from "./vault.svelte";

const mocks = vi.hoisted(() => ({
  emptyRecycleBin: vi.fn(),
  toastSuccess: vi.fn(),
  toastError: vi.fn(),
}));

vi.mock("$lib/bridge/vault", () => ({ vaultEmptyRecycleBin: mocks.emptyRecycleBin }));
vi.mock("$lib/stores/toast.svelte", () => ({
  toast: { success: mocks.toastSuccess, error: mocks.toastError },
}));

const active: EntrySummary = {
  id: "active",
  type: "login",
  title: "Active",
  subtitle: "",
  tags: [],
  favorite: false,
};
const trashed: EntrySummary = { ...active, id: "trashed", title: "Deleted", trashed: true };

beforeEach(() => {
  vi.clearAllMocks();
  recycleBin.cancel();
  vault.setMeta(null);
  vault.setEntries([active, trashed]);
  selection.selectedId = trashed.id;
});

describe("recycleBin", () => {
  it("removes trashed entries and clears their selection", async () => {
    mocks.emptyRecycleBin.mockResolvedValue({ deletedEntries: 1 });
    recycleBin.requestEmpty();

    await recycleBin.confirm();

    expect(mocks.emptyRecycleBin).toHaveBeenCalledOnce();
    expect(vault.entries).toEqual([active]);
    expect(selection.selectedId).toBeNull();
    expect(mocks.toastSuccess).toHaveBeenCalledWith("Permanently deleted 1 entry");
  });

  it("does not request confirmation for an empty bin", () => {
    vault.setEntries([active]);

    recycleBin.requestEmpty();

    expect(recycleBin.pending).toBe(false);
  });

  it("keeps entries when emptying fails", async () => {
    mocks.emptyRecycleBin.mockRejectedValue(new Error("Empty failed"));
    recycleBin.requestEmpty();

    await recycleBin.confirm();

    expect(vault.entries).toEqual([active, trashed]);
    expect(selection.selectedId).toBe(trashed.id);
    expect(mocks.toastError).toHaveBeenCalledWith("Empty failed");
  });
});
