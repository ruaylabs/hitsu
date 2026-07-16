import { fireEvent, render, screen } from "@testing-library/svelte";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { vault } from "$lib/stores/vault.svelte";
import SettingsView from "./SettingsView.svelte";

const mocks = vi.hoisted(() => ({
  import1pif: vi.fn(),
  setFoldersEnabled: vi.fn(),
}));

vi.mock("@tauri-apps/plugin-dialog", () => ({
  open: vi.fn(),
  save: vi.fn(),
}));

vi.mock("$lib/bridge/prefs", () => ({
  prefsGet: vi.fn().mockResolvedValue({
    idleLockMinutes: 5,
    clipboardClearSeconds: 15,
    foldersEnabled: false,
    recentVaults: [],
  }),
  prefsSetSecurity: vi.fn(),
  prefsSetFoldersEnabled: mocks.setFoldersEnabled,
}));

vi.mock("$lib/bridge/vault", () => ({
  vaultImport1pif: mocks.import1pif,
}));

describe("SettingsView import details", () => {
  beforeEach(() => {
    vault.setMeta({
      path: "/tmp/test.kdbx",
      name: "Test",
      itemCount: 0,
      syncProvider: "local",
      entries: [],
      folders: [],
    });
    vault.setEntries([]);
    mocks.import1pif.mockResolvedValue({
      importedItems: 1,
      importedAttachments: 0,
      skippedItems: 2,
      skippedEntries: [
        { title: "Archived login", reason: "Item is in the 1Password trash" },
        { title: "Unsupported document", reason: "The item couldn't be converted" },
      ],
      entries: [],
    });
  });

  it("enables optional folder support", async () => {
    render(SettingsView);

    const toggle = await screen.findByRole("switch", { name: "Enable folders" });
    expect(toggle).not.toBeChecked();
    await fireEvent.click(toggle);

    expect(mocks.setFoldersEnabled).toHaveBeenCalledWith(true);
  });

  it("shows skipped entry names in a simple list", async () => {
    render(SettingsView);

    await fireEvent.click(screen.getByRole("button", { name: /Import 1Password 7/ }));
    await fireEvent.click(await screen.findByRole("button", { name: "View details" }));

    expect(screen.getByRole("dialog", { name: "Entries not imported" })).toBeInTheDocument();
    const entries = screen.getAllByRole("listitem");
    expect(entries[0]).toHaveTextContent("Archived login");
    expect(entries[0]).toHaveTextContent("Item is in the 1Password trash");
    expect(entries[1]).toHaveTextContent("Unsupported document");
    expect(entries[1]).toHaveTextContent("The item couldn't be converted");
  });
});
