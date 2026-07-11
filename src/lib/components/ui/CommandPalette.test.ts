import { fireEvent, render, screen } from "@testing-library/svelte";
import { describe, expect, it, vi } from "vitest";
import CommandPalette from "./CommandPalette.svelte";

describe("CommandPalette", () => {
  it("lists and selects password entries", async () => {
    const onSelect = vi.fn();

    render(CommandPalette, { onSelect, onClose: vi.fn() });

    const passwordOption = screen.getByRole("option", { name: "Password" });
    expect(passwordOption).toBeInTheDocument();

    await fireEvent.click(passwordOption);
    expect(onSelect).toHaveBeenCalledWith("password");
  });
});
