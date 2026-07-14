import { fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { describe, expect, it, vi } from "vitest";
import type { EntrySummary } from "$lib/bridge/types";
import EntryCommandPalette from "./EntryCommandPalette.svelte";

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

  it("supports keyboard navigation and selection", async () => {
    const onSelect = vi.fn();
    render(EntryCommandPalette, { entries, onSelect, onClose: vi.fn() });
    const input = screen.getByRole("textbox", { name: "Search entries" });

    await fireEvent.keyDown(input, { key: "ArrowDown" });
    await fireEvent.keyDown(input, { key: "Enter" });

    expect(onSelect).toHaveBeenCalledWith(entries[1]);
  });
});
