import { fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { Entry } from "$lib/bridge/types";
import HistoryDialog from "./HistoryDialog.svelte";

const mocks = vi.hoisted(() => ({
  entryHistoryList: vi.fn(),
  entryHistoryGet: vi.fn(),
  entryRevealField: vi.fn(),
  entryCopyField: vi.fn(),
}));

vi.mock("$lib/bridge/entries", () => ({
  entryHistoryList: mocks.entryHistoryList,
  entryHistoryGet: mocks.entryHistoryGet,
  entryRevealField: mocks.entryRevealField,
  entryCopyField: mocks.entryCopyField,
}));

function cardRevision(version: number): Entry {
  return {
    id: "card-1",
    type: "card",
    title: version === 2 ? "Current card" : "Older card",
    subtitle: "Visa ending in 1111",
    hasPassword: false,
    hasTotp: false,
    tags: [],
    favorite: false,
    card: {
      type: "visa",
      numberMasked: "4111 •••• 1111",
      hasNumber: true,
      hasCvv: false,
      hasPin: false,
    },
    attachments: [],
    customFields: [],
    modifiedAt: `2026-07-${version === 2 ? "12" : "11"}T00:00:00Z`,
    createdAt: "2026-07-10T00:00:00Z",
    historyCount: 2,
    hasCustomIcon: false,
  };
}

beforeEach(() => {
  vi.clearAllMocks();
  mocks.entryHistoryList.mockResolvedValue([
    { version: 2, modifiedAt: "2026-07-12T00:00:00Z", title: "Current card" },
    { version: 1, modifiedAt: "2026-07-11T00:00:00Z", title: "Older card" },
  ]);
  mocks.entryHistoryGet.mockImplementation((_id: string, version: number) =>
    Promise.resolve(cardRevision(version)),
  );
  mocks.entryRevealField.mockImplementation((_id: string, _field: string, version: number) =>
    Promise.resolve(version === 2 ? "4111111111111111" : "5555555555554444"),
  );
});

afterEach(() => {
  vi.restoreAllMocks();
});

describe("HistoryDialog", () => {
  it("reveals historical card numbers only on request and hides them on revision changes", async () => {
    render(HistoryDialog, { entryId: "card-1", onclose: vi.fn() });

    expect(await screen.findByText("4111 •••• 1111")).toBeInTheDocument();
    expect(mocks.entryRevealField).not.toHaveBeenCalled();

    await fireEvent.click(screen.getByRole("button", { name: "Reveal number" }));

    expect(await screen.findByText("4111 1111 1111 1111")).toBeInTheDocument();
    expect(mocks.entryRevealField).toHaveBeenCalledWith("card-1", "cardNumber", 2);

    await fireEvent.click(screen.getByRole("button", { name: /Older card/ }));

    expect(screen.queryByText("4111 1111 1111 1111")).not.toBeInTheDocument();
    await waitFor(() => expect(mocks.entryHistoryGet).toHaveBeenCalledWith("card-1", 1));
    expect(mocks.entryRevealField).toHaveBeenCalledTimes(1);
  });
});
