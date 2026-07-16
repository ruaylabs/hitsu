import { fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { beforeEach, describe, expect, it, vi } from "vitest";
import * as entriesBridge from "$lib/bridge/entries";
import type { EntrySummary } from "$lib/bridge/types";
import EntryCommandPalette from "./EntryCommandPalette.svelte";

vi.mock("$lib/bridge/entries", async (importOriginal) => ({
  ...(await importOriginal<typeof import("$lib/bridge/entries")>()),
  entriesSearch: vi.fn(),
}));

const entriesSearchMock = vi.mocked(entriesBridge.entriesSearch);

const entries: EntrySummary[] = [
  {
    id: "one",
    type: "login",
    title: "Kagi",
    subtitle: "hello@example.com",
    tags: ["search"],
    favorite: true,
  },
  {
    id: "two",
    type: "card",
    title: "Travel card",
    subtitle: "Visa ending in 4242",
    tags: ["finance"],
    favorite: false,
  },
];

describe("EntryCommandPalette", () => {
  beforeEach(() => {
    entriesSearchMock.mockReset();
    entriesSearchMock.mockRejectedValue(new Error("backend search unavailable"));
  });

  it("filters entries across searchable fields and selects a result", async () => {
    const onSelect = vi.fn();
    render(EntryCommandPalette, { entries, onSelect, onClose: vi.fn() });

    const searchInput = screen.getByRole("textbox", { name: "Search entries" });
    await waitFor(() => expect(searchInput).toHaveFocus());
    await fireEvent.input(searchInput, {
      target: { value: "4242" },
    });

    expect(screen.queryByRole("option", { name: /Kagi/ })).not.toBeInTheDocument();
    const result = screen.getByRole("option", { name: /Travel card/ });
    await fireEvent.click(result);
    expect(onSelect).toHaveBeenCalledWith(entries[1]);
  });

  it("includes backend matches from full entry fields", async () => {
    entriesSearchMock.mockResolvedValue(["one"]);
    render(EntryCommandPalette, { entries, onSelect: vi.fn(), onClose: vi.fn() });

    await fireEvent.input(screen.getByRole("textbox", { name: "Search entries" }), {
      target: { value: "buried note" },
    });

    expect(await screen.findByRole("option", { name: /Kagi/ })).toBeInTheDocument();
    expect(screen.queryByRole("option", { name: /Travel card/ })).not.toBeInTheDocument();
  });

  it("supports Ctrl+N and Ctrl+P navigation", async () => {
    const onSelect = vi.fn();
    render(EntryCommandPalette, { entries, onSelect, onClose: vi.fn() });
    const input = screen.getByRole("textbox", { name: "Search entries" });

    await fireEvent.keyDown(input, { key: "n", ctrlKey: true });
    expect(screen.getByRole("option", { name: /Travel card/ })).toHaveAttribute(
      "aria-selected",
      "true",
    );

    await fireEvent.keyDown(input, { key: "p", ctrlKey: true });
    expect(screen.getByRole("option", { name: /Kagi/ })).toHaveAttribute("aria-selected", "true");

    await fireEvent.keyDown(input, { key: "n", ctrlKey: true });
    await fireEvent.keyDown(input, { key: "Enter" });
    expect(onSelect).toHaveBeenCalledWith(entries[1]);
  });
});
