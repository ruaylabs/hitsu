import { fireEvent, render, screen } from "@testing-library/svelte";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import PasswordField from "./PasswordField.svelte";

vi.mock("$lib/stores/security.svelte", () => ({
  security: { clipboardClearSeconds: 5 },
}));

beforeEach(() => {
  vi.useFakeTimers();
});

afterEach(() => {
  vi.useRealTimers();
});

describe("PasswordField", () => {
  it("uses the clipboard timeout when automatically hiding a secret", async () => {
    render(PasswordField, {
      label: "Password",
      reveal: vi.fn().mockResolvedValue("revealed-secret"),
      copy: vi.fn(),
    });

    await fireEvent.click(screen.getByRole("button", { name: "Reveal password" }));

    expect(screen.getByText("revealed-secret")).toBeInTheDocument();
    expect(screen.getByText("Hides in 5s")).toBeInTheDocument();
    await vi.advanceTimersByTimeAsync(6000);
    expect(screen.queryByText("revealed-secret")).not.toBeInTheDocument();
  });
});
