import { fireEvent, render, screen } from "@testing-library/svelte";
import { describe, expect, it, vi } from "vitest";
import SearchField from "./SearchField.svelte";

describe("SearchField", () => {
  it("shows the search shortcut while the field is empty", async () => {
    render(SearchField);

    expect(screen.getByText(/F$/, { selector: "kbd" })).toBeInTheDocument();
    await fireEvent.input(screen.getByRole("textbox"), { target: { value: "vault" } });

    expect(screen.queryByText(/F$/, { selector: "kbd" })).not.toBeInTheDocument();
  });

  it("opens the shared new-entry picker from the add button", async () => {
    const onCreate = vi.fn();
    render(SearchField, { onCreate });

    await fireEvent.click(screen.getByRole("button", { name: "Add entry" }));

    expect(onCreate).toHaveBeenCalledOnce();
    expect(screen.queryByRole("menu")).not.toBeInTheDocument();
  });
});
