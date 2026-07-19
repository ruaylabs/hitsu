import { fireEvent, render, screen } from "@testing-library/svelte";
import { describe, expect, it, vi } from "vitest";
import PasswordDialog from "./PasswordDialog.svelte";

function passwordInput(): HTMLInputElement {
  return screen.getByLabelText("Master password");
}

describe("PasswordDialog", () => {
  it("does not submit an empty password", async () => {
    const onconfirm = vi.fn();
    render(PasswordDialog, { onconfirm, oncancel: vi.fn() });

    expect(screen.getByRole("button", { name: "Unlock" })).toBeDisabled();
    await fireEvent.keyDown(window, { key: "Enter" });

    expect(onconfirm).not.toHaveBeenCalled();
    expect(screen.getByText("Password is required")).toBeInTheDocument();
  });

  it("requires confirmation fields to match", async () => {
    const onconfirm = vi.fn();
    render(PasswordDialog, {
      confirm: true,
      confirmLabel: "Create",
      onconfirm,
      oncancel: vi.fn(),
    });

    await fireEvent.input(passwordInput(), { target: { value: "first-password" } });
    await fireEvent.input(screen.getByLabelText("Confirm password"), {
      target: { value: "different-password" },
    });
    await fireEvent.click(screen.getByRole("button", { name: "Create" }));

    expect(onconfirm).not.toHaveBeenCalled();
    expect(screen.getByText("Passwords do not match")).toBeInTheDocument();
  });

  it("enforces the configured strength requirement", async () => {
    render(PasswordDialog, {
      confirmLabel: "Create",
      showStrength: true,
      minStrength: 3,
      onconfirm: vi.fn(),
      oncancel: vi.fn(),
    });
    const submit = screen.getByRole("button", { name: "Create" });

    await fireEvent.input(passwordInput(), { target: { value: "abcdefgh" } });
    expect(submit).toBeDisabled();

    await fireEvent.input(passwordInput(), {
      target: { value: "Correct-Horse-Battery-Staple-42" },
    });
    expect(submit).toBeEnabled();
  });

  it("clears local validation errors after editing", async () => {
    render(PasswordDialog, {
      confirm: true,
      confirmLabel: "Create",
      onconfirm: vi.fn(),
      oncancel: vi.fn(),
    });
    const confirmation = screen.getByLabelText("Confirm password");

    await fireEvent.input(passwordInput(), { target: { value: "first-password" } });
    await fireEvent.input(confirmation, { target: { value: "different-password" } });
    await fireEvent.click(screen.getByRole("button", { name: "Create" }));
    expect(screen.getByText("Passwords do not match")).toBeInTheDocument();

    await fireEvent.input(confirmation, { target: { value: "first-password" } });
    expect(screen.queryByText("Passwords do not match")).not.toBeInTheDocument();
  });

  it("submits a valid password with Enter", async () => {
    const onconfirm = vi.fn();
    render(PasswordDialog, { onconfirm, oncancel: vi.fn() });

    await fireEvent.input(passwordInput(), { target: { value: "valid-password" } });
    await fireEvent.keyDown(window, { key: "Enter" });

    expect(onconfirm).toHaveBeenCalledOnce();
    expect(onconfirm).toHaveBeenCalledWith("valid-password");
  });

  it("shows the vault path when provided", () => {
    render(PasswordDialog, {
      vaultPath: "/home/user/vaults/personal.kdbx",
      onconfirm: vi.fn(),
      oncancel: vi.fn(),
    });

    // Substring match: the rendered text starts with an invisible &lrm; mark.
    expect(screen.getByText(/\/home\/user\/vaults\/personal\.kdbx/)).toBeInTheDocument();
  });

  it("invokes cancel from the footer", async () => {
    const oncancel = vi.fn();
    render(PasswordDialog, { onconfirm: vi.fn(), oncancel });

    await fireEvent.click(screen.getByText("Cancel"));

    expect(oncancel).toHaveBeenCalledOnce();
  });
});
