import { fireEvent, render, screen } from "@testing-library/svelte";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { selection } from "$lib/stores/selection.svelte";
import SearchField from "./SearchField.svelte";

beforeEach(() => {
  selection.search = "";
  selection.filter = { kind: "all" };
});

describe("SearchField", () => {
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
    expect(screen.getByText("Favorites")).toBeInTheDocument();

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
