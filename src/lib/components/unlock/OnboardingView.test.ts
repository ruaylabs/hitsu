import { fireEvent, render, screen } from "@testing-library/svelte";
import { beforeEach, describe, expect, it, vi } from "vitest";
import OnboardingView from "./OnboardingView.svelte";

const mocks = vi.hoisted(() => ({ prefsGet: vi.fn(), vaultOpen: vi.fn() }));

vi.mock("$lib/bridge/prefs", () => ({ prefsGet: mocks.prefsGet }));
vi.mock("$lib/bridge/vault", () => ({ vaultOpen: mocks.vaultOpen }));
vi.mock("@tauri-apps/plugin-dialog", () => ({ open: vi.fn(), save: vi.fn() }));

beforeEach(() => {
  vi.clearAllMocks();
  mocks.vaultOpen.mockRejectedValue(new Error("Invalid master password"));
  mocks.prefsGet.mockResolvedValue({
    lastVault: "/vaults/main.kdbx",
    recentVaults: ["/vaults/main.kdbx", "/vaults/work.kdbx"],
    idleLockMinutes: 5,
    clipboardClearSeconds: 15,
    foldersEnabled: false,
    browserIntegrationEnabled: false,
    kdfUpgradeDismissedVaults: [],
  });
});

describe("OnboardingView", () => {
  it("lists unique recent vaults and opens the selected unlock dialog", async () => {
    render(OnboardingView);

    expect(await screen.findByText("Recent vaults")).toBeInTheDocument();
    expect(screen.getAllByRole("button", { name: "main.kdbx" })).toHaveLength(1);
    await fireEvent.click(screen.getByRole("button", { name: "work.kdbx" }));

    expect(screen.getByRole("dialog", { name: "Unlock vault" })).toBeInTheDocument();
    expect(screen.getByText(/\/vaults\/work\.kdbx/)).toBeInTheDocument();
  });

  it("keeps a failed unlock in context", async () => {
    render(OnboardingView);

    await fireEvent.click(await screen.findByRole("button", { name: "work.kdbx" }));
    const password = screen.getByLabelText("Master password");
    await fireEvent.input(password, { target: { value: "incorrect-password" } });
    await fireEvent.click(screen.getByRole("button", { name: "Unlock" }));

    expect(await screen.findByText("Invalid master password")).toBeInTheDocument();
    expect(password).toHaveValue("incorrect-password");
    expect(screen.getByRole("dialog", { name: "Unlock vault" })).toBeInTheDocument();
  });
});
