import { fireEvent, render, screen } from "@testing-library/svelte";
import { beforeEach, describe, expect, it, vi } from "vitest";
import OnboardingView from "./OnboardingView.svelte";

const mocks = vi.hoisted(() => ({ prefsGet: vi.fn() }));

vi.mock("$lib/bridge/prefs", () => ({ prefsGet: mocks.prefsGet }));
vi.mock("@tauri-apps/plugin-dialog", () => ({ open: vi.fn(), save: vi.fn() }));

beforeEach(() => {
  vi.clearAllMocks();
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
});
