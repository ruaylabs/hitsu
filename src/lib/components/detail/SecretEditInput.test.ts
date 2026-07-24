import { fireEvent, render, screen } from "@testing-library/svelte";
import { describe, expect, it } from "vitest";
import SecretEditInput from "./SecretEditInput.svelte";

describe("SecretEditInput", () => {
  it("is masked by default and toggles visibility", async () => {
    render(SecretEditInput, { value: "secret", label: "password" });
    const input = screen.getByLabelText("password");

    expect(input).toHaveAttribute("type", "password");
    await fireEvent.click(screen.getByRole("button", { name: "Reveal password" }));
    expect(input).toHaveAttribute("type", "text");
    await fireEvent.click(screen.getByRole("button", { name: "Hide password" }));
    expect(input).toHaveAttribute("type", "password");
  });

  it("sanitizes numeric secrets", async () => {
    render(SecretEditInput, {
      value: "",
      label: "PIN",
      sanitize: (value: string) => value.replace(/\D/g, "").slice(0, 4),
    });
    const input = screen.getByLabelText("PIN");

    await fireEvent.input(input, { target: { value: "12ab345" } });
    expect(input).toHaveValue("1234");
  });
});
