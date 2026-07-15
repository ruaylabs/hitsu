import { fireEvent, render, screen } from "@testing-library/svelte";
import { describe, expect, it } from "vitest";
import SearchField from "./SearchField.svelte";

describe("SearchField", () => {
  it("offers every entry type from the add button", async () => {
    render(SearchField);

    await fireEvent.click(screen.getByRole("button", { name: "Add entry" }));

    expect(screen.getByRole("menuitem", { name: "Login" })).toBeInTheDocument();
    expect(screen.getByRole("menuitem", { name: "Password" })).toBeInTheDocument();
    expect(screen.getByRole("menuitem", { name: "Note" })).toBeInTheDocument();
    expect(screen.getByRole("menuitem", { name: "Identity" })).toBeInTheDocument();
    expect(screen.getByRole("menuitem", { name: "Card" })).toBeInTheDocument();
    expect(screen.getByRole("menuitem", { name: "Software License" })).toBeInTheDocument();
    expect(screen.getByRole("menuitem", { name: "Passport" })).toBeInTheDocument();
  });
});
