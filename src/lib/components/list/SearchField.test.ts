import { fireEvent, render, screen } from "@testing-library/svelte";
import { describe, expect, it, vi } from "vitest";
import SearchField from "./SearchField.svelte";

describe("SearchField", () => {
  it("opens the shared new-entry picker from the add button", async () => {
    const onCreate = vi.fn();
    render(SearchField, { onCreate });

    await fireEvent.click(screen.getByRole("button", { name: "Add entry" }));

    expect(onCreate).toHaveBeenCalledOnce();
    expect(screen.queryByRole("menu")).not.toBeInTheDocument();
  });
});
