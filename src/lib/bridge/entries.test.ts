import { invoke } from "@tauri-apps/api/core";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { entriesSearch, entryCreate, entryEditPayload, entryMove, entryUpdate } from "./entries";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

const invokeMock = vi.mocked(invoke);

describe("entries bridge", () => {
  beforeEach(() => {
    invokeMock.mockReset();
  });

  it("passes the create draft directly over IPC", async () => {
    invokeMock.mockResolvedValue({});
    const draft = { title: "New login", username: "alice" };

    await entryCreate("login", draft);

    expect(invokeMock).toHaveBeenCalledWith("entry_create", { itemType: "login", draft });
  });

  it("searches entry fields in the backend", async () => {
    invokeMock.mockResolvedValue(["entry-1"]);

    await expect(entriesSearch("recovery note")).resolves.toEqual(["entry-1"]);

    expect(invokeMock).toHaveBeenCalledWith("entries_search", { query: "recovery note" });
  });

  it("requests one edit payload for all protected fields", async () => {
    invokeMock.mockResolvedValue({});

    await entryEditPayload("entry-1");

    expect(invokeMock).toHaveBeenCalledWith("entry_edit_payload", { id: "entry-1" });
  });

  it("moves an entry to a folder", async () => {
    invokeMock.mockResolvedValue({});

    await entryMove("entry-1", "folder-1");

    expect(invokeMock).toHaveBeenCalledWith("entry_move", {
      id: "entry-1",
      folderId: "folder-1",
    });
  });

  it("passes only the supplied update fields over IPC", async () => {
    invokeMock.mockResolvedValue({});

    await entryUpdate("entry-1", { title: "Updated" });

    expect(invokeMock).toHaveBeenCalledWith("entry_update", {
      id: "entry-1",
      patch: { title: "Updated" },
    });
  });
});
