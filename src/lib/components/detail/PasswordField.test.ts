import { fireEvent, render, screen } from "@testing-library/svelte";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { toast } from "$lib/stores/toast.svelte";
import PasswordField from "./PasswordField.svelte";

vi.mock("$lib/stores/security.svelte", () => ({
  security: { clipboardClearSeconds: 5 },
}));

beforeEach(() => {
  vi.useFakeTimers();
});

afterEach(() => {
  for (const item of toast.all) toast.dismiss(item.id);
  vi.useRealTimers();
});

describe("PasswordField", () => {
  it("announces a successful copy", async () => {
    const copy = vi.fn().mockResolvedValue(undefined);
    render(PasswordField, {
      label: "Password",
      reveal: vi.fn(),
      copy,
    });

    await fireEvent.click(screen.getByRole("button", { name: "Copy password" }));

    expect(copy).toHaveBeenCalledOnce();
    expect(toast.all.at(-1)).toMatchObject({ kind: "success", message: "Password copied" });
  });

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
