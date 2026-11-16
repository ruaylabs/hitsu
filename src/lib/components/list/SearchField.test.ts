import { fireEvent, render, screen } from "@testing-library/svelte";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { selection } from "$lib/stores/selection.svelte";
import SearchField from "./SearchField.svelte";

beforeEach(() => {
  selection.search = "";
  selection.filter = { kind: "all" };
});

afterEach(() => vi.restoreAllMocks());

describe("SearchField", () => {
  it.each(["MacIntel", "Linux x86_64"])("limits header dragging to macOS (%s)", (platform) => {
    vi.spyOn(navigator, "platform", "get").mockReturnValue(platform);
    const { container } = render(SearchField);

    expect(container.querySelectorAll("[data-tauri-drag-region]")).toHaveLength(
      platform === "MacIntel" ? 2 : 0,
    );
    expect(screen.getByRole("textbox")).not.toHaveAttribute("data-tauri-drag-region");
    expect(screen.getByRole("button", { name: "Add entry" })).not.toHaveAttribute(
      "data-tauri-drag-region",
    );
  });

  it("shows the search shortcut while the field is empty", async () => {
    render(SearchField);

    expect(screen.getByText(/F$/, { selector: "kbd" })).toBeInTheDocument();
    await fireEvent.input(screen.getByRole("textbox"), { target: { value: "vault" } });

    expect(screen.queryByText(/F$/, { selector: "kbd" })).not.toBeInTheDocument();
  });

  it("shows and removes the active scope while keeping global search available", async () => {
    selection.filter = { kind: "favorites" };
    render(SearchField);

    const search = screen.getByRole("textbox", { name: "Search in Favorites…" });
    expect(search).toHaveAttribute("placeholder", "Search in Favorites…");
    expect(screen.getByText("Favorites", { selector: ".scope-chip > span" })).toBeInTheDocument();

    await fireEvent.input(search, { target: { value: "vault" } });
    await fireEvent.click(screen.getByRole("button", { name: "Search all items" }));

    expect(selection.filter).toEqual({ kind: "all" });
    expect(screen.getByRole("textbox", { name: "Search all items…" })).toBeInTheDocument();
  });

  it("opens the shared new-entry picker from the add button", async () => {
    const onCreate = vi.fn();
    render(SearchField, { onCreate });

    await fireEvent.click(screen.getByRole("button", { name: "Add entry" }));

    expect(onCreate).toHaveBeenCalledOnce();
    expect(screen.queryByRole("menu")).not.toBeInTheDocument();
  });
});
