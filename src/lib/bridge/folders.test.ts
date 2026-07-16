import { invoke } from "@tauri-apps/api/core";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { folderCreate, folderRename } from "./folders";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

const invokeMock = vi.mocked(invoke);

describe("folders bridge", () => {
  beforeEach(() => invokeMock.mockReset());

  it("creates a folder under the selected parent", async () => {
    invokeMock.mockResolvedValue({});

    await folderCreate("parent-1", "Clients");

    expect(invokeMock).toHaveBeenCalledWith("folder_create", {
      parentId: "parent-1",
      name: "Clients",
    });
  });

  it("renames a folder", async () => {
    invokeMock.mockResolvedValue({});

    await folderRename("folder-1", "Projects");

    expect(invokeMock).toHaveBeenCalledWith("folder_rename", {
      id: "folder-1",
      name: "Projects",
    });
  });
});
