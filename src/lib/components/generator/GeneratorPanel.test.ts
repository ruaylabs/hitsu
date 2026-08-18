import { fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { clipboard } from "$lib/stores/clipboard.svelte";
import GeneratorPanel from "./GeneratorPanel.svelte";

const mocks = vi.hoisted(() => ({ generatePassword: vi.fn() }));

vi.mock("$lib/bridge/generator", () => ({
  generatePassword: mocks.generatePassword,
}));

const copyPlainMock = vi.spyOn(clipboard, "copyPlain");

beforeEach(() => {
  mocks.generatePassword.mockReset();
  copyPlainMock.mockReset();
  localStorage.clear();
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

describe("GeneratorPanel actions", () => {
  it("copies the generated password", async () => {
    mocks.generatePassword.mockResolvedValue("generated-password");
    render(GeneratorPanel, { oncancel: vi.fn(), onUse: vi.fn() });

    await screen.findByText("generated-password");
    await fireEvent.click(screen.getByRole("button", { name: "Copy password" }));

    await waitFor(() => expect(copyPlainMock).toHaveBeenCalledWith("generated-password"));
  });

  it("shows a strength meter for the generated password", async () => {
    mocks.generatePassword.mockResolvedValue("generated-password");
    render(GeneratorPanel, { oncancel: vi.fn(), onUse: vi.fn() });

    const status = await screen.findByRole("status", { name: /Password strength/ });
    expect(status).toHaveTextContent("Strength:");
  });

  it("restores the last used options", async () => {
    localStorage.setItem(
      "hitsu:generator-options",
      JSON.stringify({
        length: 42,
        uppercase: false,
        lowercase: true,
        digits: true,
        symbols: true,
        excludeLookalikes: false,
      }),
    );
    mocks.generatePassword.mockResolvedValue("generated-password");
    render(GeneratorPanel, { oncancel: vi.fn(), onUse: vi.fn() });

    await waitFor(() =>
      expect(mocks.generatePassword).toHaveBeenCalledWith(
        expect.objectContaining({ length: 42, uppercase: false, symbols: true }),
      ),
    );
    const lengthSlider = screen.getByRole("slider");
    expect(lengthSlider).toHaveValue("42");
  });

  it("persists options when they change", async () => {
    mocks.generatePassword.mockResolvedValue("generated-password");
    render(GeneratorPanel, { oncancel: vi.fn(), onUse: vi.fn() });
    await screen.findByText("generated-password");

    await fireEvent.click(screen.getByRole("checkbox", { name: "Symbols" }));

    await waitFor(() =>
      expect(JSON.parse(localStorage.getItem("hitsu:generator-options") ?? "{}")).toMatchObject({
        symbols: true,
      }),
    );
  });
});
