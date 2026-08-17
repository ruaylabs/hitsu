import { fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { VaultMeta } from "$lib/bridge/types";
import * as vaultBridge from "$lib/bridge/vault";
import { vault } from "$lib/stores/vault.svelte";
import VaultSelector from "./VaultSelector.svelte";

const mocks = vi.hoisted(() => ({ prefsGet: vi.fn() }));

vi.mock("$lib/bridge/prefs", () => ({ prefsGet: mocks.prefsGet }));
vi.mock("@tauri-apps/plugin-dialog", () => ({ open: vi.fn(), save: vi.fn() }));

const meta: VaultMeta = {
  path: "/vaults/main.kdbx",
  name: "Main",
  itemCount: 0,
  syncProvider: "local",
  entries: [],
  folders: [],
};

beforeEach(() => {
  vi.restoreAllMocks();
  mocks.prefsGet.mockResolvedValue({
    lastVault: "/vaults/main.kdbx",
    recentVaults: ["/vaults/main.kdbx", "/vaults/work.kdbx", "/vaults/old.kdbx"],
    idleLockMinutes: 5,
    clipboardClearSeconds: 15,
    foldersEnabled: false,
    browserIntegrationEnabled: false,
    kdfUpgradeDismissedVaults: [],
  });
  vi.spyOn(vaultBridge, "vaultLock").mockResolvedValue(undefined);
  vault.unlock();
  vault.setMeta(meta);
});

describe("VaultSelector", () => {
  it("shows the current vault and lists other recent vaults", async () => {
    render(VaultSelector);

    const button = await screen.findByRole("button", { name: /Main/ });
    expect(button).toHaveAttribute("aria-expanded", "false");

    await fireEvent.click(button);

    expect(button).toHaveAttribute("aria-expanded", "true");
    expect(screen.getByRole("menuitem", { name: "Lock vault" })).toBeInTheDocument();
    expect(screen.getByRole("menuitem", { name: "Open other vault…" })).toBeInTheDocument();
    expect(screen.getByRole("menuitem", { name: "work.kdbx" })).toBeInTheDocument();
    expect(screen.getByRole("menuitem", { name: "old.kdbx" })).toBeInTheDocument();
    // The open vault is not offered as a switch target.
    expect(screen.queryByRole("menuitem", { name: "main.kdbx" })).not.toBeInTheDocument();
  });

  it("locks and switches to the selected vault", async () => {
    render(VaultSelector);

    await fireEvent.click(await screen.findByRole("button", { name: /Main/ }));
    await fireEvent.click(screen.getByRole("menuitem", { name: "work.kdbx" }));

    expect(vaultBridge.vaultLock).toHaveBeenCalledOnce();
    await waitFor(() => expect(vault.locked).toBe(true));
    expect(vault.meta).toMatchObject({ path: "/vaults/work.kdbx", name: "work.kdbx" });
  });

  it("closes the menu on escape and outside clicks", async () => {
    render(VaultSelector);

    const button = await screen.findByRole("button", { name: /Main/ });
    await fireEvent.click(button);
    expect(screen.getByRole("menu")).toBeInTheDocument();

    await fireEvent.keyDown(window, { key: "Escape" });
    expect(screen.queryByRole("menu")).not.toBeInTheDocument();

    await fireEvent.click(button);
    await fireEvent.pointerDown(document.body);
    expect(screen.queryByRole("menu")).not.toBeInTheDocument();
  });
});
