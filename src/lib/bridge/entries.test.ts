import { invoke } from "@tauri-apps/api/core";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { entryUpdate } from "./entries";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

const invokeMock = vi.mocked(invoke);

describe("entries bridge", () => {
  beforeEach(() => {
    invokeMock.mockReset();
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
