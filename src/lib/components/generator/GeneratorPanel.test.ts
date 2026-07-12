import { render, screen } from "@testing-library/svelte";
import { describe, expect, it, vi } from "vitest";
import GeneratorPanel from "./GeneratorPanel.svelte";

const mocks = vi.hoisted(() => ({ generatePassword: vi.fn() }));

vi.mock("$lib/bridge/generator", () => ({
  generatePassword: mocks.generatePassword,
}));

describe("GeneratorPanel errors", () => {
  it("shows generation failures without allowing the error to be used", async () => {
    mocks.generatePassword.mockRejectedValue(new Error("Generator unavailable"));

    render(GeneratorPanel, { oncancel: vi.fn(), onUse: vi.fn() });

    expect(await screen.findByText("Failed to generate a password")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Use this" })).toBeDisabled();
  });
});
