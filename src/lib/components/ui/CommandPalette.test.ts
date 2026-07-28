import { fireEvent, render, screen } from "@testing-library/svelte";
import { describe, expect, it, vi } from "vitest";
import CommandPalette from "./CommandPalette.svelte";

describe("CommandPalette", () => {
  it("lists and selects password entries", async () => {
    const onSelect = vi.fn();

    render(CommandPalette, { onSelect, onClose: vi.fn() });

    const passwordOption = screen.getByRole("option", { name: "Password" });
    expect(passwordOption).toHaveAccessibleDescription(
      "Standalone password or secret without a username",
    );

    await fireEvent.click(passwordOption);
    expect(onSelect).toHaveBeenCalledWith("password");
  });

  it("matches entry-type descriptions", async () => {
    render(CommandPalette, { onSelect: vi.fn(), onClose: vi.fn() });

    await fireEvent.input(screen.getByRole("textbox"), {
      target: { value: "travel document" },
    });

    expect(screen.getByRole("option", { name: "Passport" })).toBeInTheDocument();
    expect(screen.queryByRole("option", { name: "Login" })).not.toBeInTheDocument();
  });
});
