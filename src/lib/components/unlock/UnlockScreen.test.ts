import { fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { beforeEach, describe, expect, it, vi } from "vitest";
import UnlockScreen from "./UnlockScreen.svelte";

const mocks = vi.hoisted(() => ({
  biometricStatus: vi.fn(),
  open: vi.fn(),
  unlockWithBiometric: vi.fn(),
}));

vi.mock("$lib/bridge/biometric", () => ({ biometricStatus: mocks.biometricStatus }));
vi.mock("$lib/stores/vault.svelte", () => ({
  vault: {
    open: mocks.open,
    unlockWithBiometric: mocks.unlockWithBiometric,
  },
}));

beforeEach(() => {
  vi.clearAllMocks();
  mocks.biometricStatus.mockResolvedValue({ available: true, enabled: true });
  mocks.open.mockResolvedValue(undefined);
  mocks.unlockWithBiometric.mockResolvedValue(undefined);
});

describe("UnlockScreen", () => {
  it("automatically requests Touch ID at startup when enabled", async () => {
    const onunlock = vi.fn();
    render(UnlockScreen, {
      path: "/vaults/main.kdbx",
      title: "Unlock vault",
      autoTouchId: true,
      onunlock,
    });

    await waitFor(() => expect(onunlock).toHaveBeenCalledOnce());
    expect(mocks.unlockWithBiometric).toHaveBeenCalledExactlyOnceWith("/vaults/main.kdbx");
  });

  it("does not automatically request Touch ID when disabled or unavailable", async () => {
    mocks.biometricStatus.mockResolvedValueOnce({ available: true, enabled: false });
    const view = render(UnlockScreen, {
      path: "/vaults/main.kdbx",
      title: "Unlock vault",
      autoTouchId: true,
    });

    await waitFor(() => expect(mocks.biometricStatus).toHaveBeenCalledOnce());
    expect(mocks.unlockWithBiometric).not.toHaveBeenCalled();
    view.unmount();

    mocks.biometricStatus.mockResolvedValueOnce({ available: false, enabled: true });
    render(UnlockScreen, {
      path: "/vaults/main.kdbx",
      title: "Unlock vault",
      autoTouchId: true,
    });

    await waitFor(() => expect(mocks.biometricStatus).toHaveBeenCalledTimes(2));
    expect(mocks.unlockWithBiometric).not.toHaveBeenCalled();
  });

  it("falls back to the password after an automatic scan is canceled without retrying", async () => {
    mocks.unlockWithBiometric.mockRejectedValueOnce(
      Object.assign(new Error("Touch ID was canceled."), { kind: "biometric_canceled" }),
    );
    render(UnlockScreen, {
      path: "/vaults/main.kdbx",
      title: "Unlock vault",
      autoTouchId: true,
    });

    await waitFor(() => expect(mocks.unlockWithBiometric).toHaveBeenCalledOnce());
    await waitFor(() => expect(screen.getByLabelText("Master password")).toHaveFocus());
    expect(screen.getByLabelText("Master password")).toBeEnabled();
    expect(screen.queryByText("Touch ID was canceled.")).not.toBeInTheDocument();
    await fireEvent.click(screen.getByRole("button", { name: "Unlock with Touch ID" }));
    expect(mocks.unlockWithBiometric).toHaveBeenCalledTimes(2);
  });

  it("offers Touch ID when it is enabled for the vault", async () => {
    const onunlock = vi.fn();
    render(UnlockScreen, {
      path: "/vaults/main.kdbx",
      title: "Unlock vault",
      onunlock,
    });

    await fireEvent.click(await screen.findByRole("button", { name: "Unlock with Touch ID" }));

    expect(mocks.unlockWithBiometric).toHaveBeenCalledWith("/vaults/main.kdbx");
    expect(onunlock).toHaveBeenCalledOnce();
  });

  it("keeps master-password unlock available after Touch ID is canceled", async () => {
    mocks.unlockWithBiometric.mockRejectedValueOnce(
      Object.assign(new Error("Touch ID was canceled."), { kind: "biometric_canceled" }),
    );
    render(UnlockScreen, { path: "/vaults/main.kdbx", title: "Unlock vault" });

    await fireEvent.click(await screen.findByRole("button", { name: "Unlock with Touch ID" }));

    await waitFor(() => expect(mocks.unlockWithBiometric).toHaveBeenCalledOnce());
    const password = screen.getByLabelText("Master password");
    expect(password).toBeEnabled();
    await waitFor(() => expect(password).toHaveFocus());
    expect(screen.queryByText("Touch ID was canceled.")).not.toBeInTheDocument();
  });

  it("falls back immediately and removes a stale Touch ID action", async () => {
    mocks.biometricStatus
      .mockResolvedValueOnce({ available: true, enabled: true })
      .mockResolvedValueOnce({ available: true, enabled: false });
    mocks.unlockWithBiometric.mockRejectedValueOnce(
      new Error("The saved password no longer unlocks this vault. Use your master password."),
    );
    render(UnlockScreen, { path: "/vaults/main.kdbx", title: "Unlock vault" });

    await fireEvent.click(await screen.findByRole("button", { name: "Unlock with Touch ID" }));

    expect(
      await screen.findByText(
        "The saved password no longer unlocks this vault. Use your master password.",
      ),
    ).toBeInTheDocument();
    expect(screen.getByLabelText("Master password")).toBeEnabled();
    await waitFor(() =>
      expect(
        screen.queryByRole("button", { name: "Unlock with Touch ID" }),
      ).not.toBeInTheDocument(),
    );
  });

  it("hides Touch ID when the backend reports it unavailable", async () => {
    mocks.biometricStatus.mockResolvedValueOnce({ available: false, enabled: false });
    render(UnlockScreen, { path: "/vaults/main.kdbx", title: "Unlock vault" });

    await waitFor(() => expect(mocks.biometricStatus).toHaveBeenCalledOnce());
    expect(screen.queryByRole("button", { name: "Unlock with Touch ID" })).not.toBeInTheDocument();
  });
});
