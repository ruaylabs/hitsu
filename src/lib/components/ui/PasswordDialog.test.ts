import { fireEvent, render, screen, waitFor } from "@testing-library/svelte";
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

    expect(screen.getByText(/Create is disabled until password strength is Good/)).toBeVisible();
    await fireEvent.input(passwordInput(), { target: { value: "abcdefgh" } });
    expect(submit).toBeDisabled();
    expect(screen.getByRole("status", { name: "Password strength: Very weak" })).toBeVisible();

    await fireEvent.input(passwordInput(), {
      target: { value: "Correct-Horse-Battery-Staple-42" },
    });
    expect(submit).toBeEnabled();
    expect(screen.getByRole("status", { name: "Password strength: Strong" })).toBeVisible();
  });

  it("explains that master passwords cannot be recovered", () => {
    render(PasswordDialog, {
      confirm: true,
      confirmLabel: "Create",
      showStrength: true,
      showRecoveryWarning: true,
      onconfirm: vi.fn(),
      oncancel: vi.fn(),
    });

    expect(screen.getByRole("note")).toHaveTextContent(
      "Hitsu cannot recover this password if you forget it.",
    );
    expect(screen.getByText(/Store a backup in another secure location/)).toBeVisible();
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

  it("prevents dismissal and duplicate submissions while pending", async () => {
    let finish: () => void = () => {};
    const pending = new Promise<void>((resolve) => {
      finish = resolve;
    });
    const onconfirm = vi.fn(() => pending);
    const oncancel = vi.fn();
    render(PasswordDialog, { confirmLabel: "Unlock", onconfirm, oncancel });

    await fireEvent.input(passwordInput(), { target: { value: "valid-password" } });
    await fireEvent.click(screen.getByRole("button", { name: "Unlock" }));

    expect(screen.getByRole("button", { name: "Unlocking…" })).toBeDisabled();
    expect(passwordInput()).toBeDisabled();
    for (const cancel of screen.getAllByRole("button", { name: "Cancel" })) {
      expect(cancel).toBeDisabled();
    }

    await fireEvent.keyDown(window, { key: "Enter" });
    await fireEvent.keyDown(window, { key: "Escape" });
    expect(onconfirm).toHaveBeenCalledOnce();
    expect(oncancel).not.toHaveBeenCalled();

    finish();
    await waitFor(() => expect(screen.getByRole("button", { name: "Unlock" })).toBeEnabled());
  });

  it("keeps the password and shows rejected operations inline", async () => {
    render(PasswordDialog, {
      onconfirm: vi.fn().mockRejectedValue(new Error("Invalid master password")),
      oncancel: vi.fn(),
    });

    await fireEvent.input(passwordInput(), { target: { value: "incorrect-password" } });
    await fireEvent.click(screen.getByRole("button", { name: "Unlock" }));

    expect(await screen.findByText("Invalid master password")).toBeInTheDocument();
    expect(passwordInput()).toHaveValue("incorrect-password");
    expect(screen.getByRole("dialog", { name: "Enter master password" })).toBeInTheDocument();
  });

  it("reveals and masks password fields independently", async () => {
    render(PasswordDialog, {
      confirm: true,
      onconfirm: vi.fn(),
      oncancel: vi.fn(),
    });
    const master = passwordInput();
    const confirmation = screen.getByLabelText("Confirm password");

    expect(master).toHaveAttribute("type", "password");
    expect(confirmation).toHaveAttribute("type", "password");

    await fireEvent.click(screen.getByRole("button", { name: "Show master password" }));
    expect(master).toHaveAttribute("type", "text");
    expect(confirmation).toHaveAttribute("type", "password");

    await fireEvent.click(screen.getByRole("button", { name: "Show confirmation password" }));
    expect(confirmation).toHaveAttribute("type", "text");
  });

  it("warns when Caps Lock is active in either password field", async () => {
    render(PasswordDialog, {
      confirm: true,
      onconfirm: vi.fn(),
      oncancel: vi.fn(),
    });
    const master = passwordInput();
    master.focus();
    const capsEvent = new KeyboardEvent("keydown", { key: "A", bubbles: true });
    vi.spyOn(capsEvent, "getModifierState").mockReturnValue(true);
    await fireEvent(master, capsEvent);

    expect(screen.getByRole("status")).toHaveTextContent("Caps Lock is on");

    const confirmation = screen.getByLabelText("Confirm password");
    confirmation.focus();
    const confirmCapsEvent = new KeyboardEvent("keydown", { key: "A", bubbles: true });
    vi.spyOn(confirmCapsEvent, "getModifierState").mockReturnValue(true);
    await fireEvent(confirmation, confirmCapsEvent);

    expect(screen.getByRole("status")).toHaveTextContent("Caps Lock is on");
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
