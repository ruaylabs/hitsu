import { fireEvent, render, screen } from "@testing-library/svelte";
import { beforeEach, describe, expect, it, vi } from "vitest";
import GeneratorPanel from "./GeneratorPanel.svelte";

const mocks = vi.hoisted(() => ({ generatePassword: vi.fn() }));

vi.mock("$lib/bridge/generator", () => ({
  generatePassword: mocks.generatePassword,
}));

beforeEach(() => {
  mocks.generatePassword.mockReset();
});

describe("GeneratorPanel errors", () => {
  it("shows generation failures without allowing the error to be used", async () => {
    mocks.generatePassword.mockRejectedValue(new Error("Generator unavailable"));

    render(GeneratorPanel, { oncancel: vi.fn(), onUse: vi.fn() });

    expect(await screen.findByRole("alert")).toHaveTextContent("Generator unavailable");
    expect(screen.getByRole("button", { name: "Use this" })).toBeDisabled();
  });

  it("explains that at least one character type is required", async () => {
    mocks.generatePassword.mockResolvedValue("generated-password");
    render(GeneratorPanel, { oncancel: vi.fn(), onUse: vi.fn() });

    await fireEvent.click(screen.getByRole("checkbox", { name: "Uppercase" }));
    await fireEvent.click(screen.getByRole("checkbox", { name: "Lowercase" }));
    await fireEvent.click(screen.getByRole("checkbox", { name: "Digits" }));

    expect(await screen.findByRole("alert")).toHaveTextContent(
      "Select at least one character type",
    );
    expect(screen.getByRole("button", { name: "Use this" })).toBeDisabled();
  });
});
