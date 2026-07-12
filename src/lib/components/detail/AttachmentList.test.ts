import { fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { AttachmentMeta } from "$lib/bridge/types";
import AttachmentList from "./AttachmentList.svelte";

const mocks = vi.hoisted(() => ({
  add: vi.fn(),
  remove: vi.fn(),
  save: vi.fn(),
  toastError: vi.fn(),
  toastSuccess: vi.fn(),
}));

vi.mock("$lib/bridge/entries", () => ({
  entryAttachmentAdd: mocks.add,
  entryAttachmentRemove: mocks.remove,
  entryAttachmentSave: mocks.save,
}));

vi.mock("$lib/stores/toast.svelte", () => ({
  toast: {
    error: mocks.toastError,
    success: mocks.toastSuccess,
  },
}));

const attachment: AttachmentMeta = {
  id: "recovery.txt",
  name: "recovery.txt",
  sizeBytes: 12,
};

beforeEach(() => vi.clearAllMocks());

describe("AttachmentList errors", () => {
  it("reports upload failures without notifying the parent", async () => {
    const onchange = vi.fn();
    mocks.add.mockRejectedValue(new Error("Upload failed"));
    const { container } = render(AttachmentList, {
      entryId: "entry-1",
      attachments: [],
      onchange,
    });
    const input = container.querySelector<HTMLInputElement>('input[type="file"]');
    if (!input) throw new Error("file input was not rendered");

    await fireEvent.change(input, {
      target: { files: [new File(["recovery"], "recovery.txt", { type: "text/plain" })] },
    });

    await waitFor(() =>
      expect(mocks.toastError).toHaveBeenCalledWith(
        "Failed to add attachment: Error: Upload failed",
      ),
    );
    expect(onchange).not.toHaveBeenCalled();
  });

  it("keeps the attachment visible when removal fails", async () => {
    const onchange = vi.fn();
    mocks.remove.mockRejectedValue(new Error("Removal failed"));
    render(AttachmentList, {
      entryId: "entry-1",
      attachments: [attachment],
      onchange,
    });

    await fireEvent.click(screen.getByRole("button", { name: "Remove recovery.txt" }));
    await fireEvent.click(screen.getByRole("button", { name: "Remove" }));

    await waitFor(() =>
      expect(mocks.toastError).toHaveBeenCalledWith(
        "Failed to remove recovery.txt: Error: Removal failed",
      ),
    );
    expect(screen.getByText("recovery.txt")).toBeInTheDocument();
    expect(onchange).not.toHaveBeenCalled();
  });
});
