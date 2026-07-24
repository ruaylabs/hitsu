import { fireEvent, render, screen } from "@testing-library/svelte";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { recycleBin } from "$lib/stores/recycleBin.svelte";
import { vault } from "$lib/stores/vault.svelte";
import SettingsView from "./SettingsView.svelte";

const mocks = vi.hoisted(() => ({
  import1pif: vi.fn(),
  setFoldersEnabled: vi.fn(),
  emptyRecycleBin: vi.fn(),
  toastSuccess: vi.fn(),
  toastError: vi.fn(),
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
  vaultEmptyRecycleBin: mocks.emptyRecycleBin,
}));

vi.mock("$lib/stores/toast.svelte", () => ({
  toast: { success: mocks.toastSuccess, error: mocks.toastError },
}));

describe("SettingsView", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    recycleBin.cancel();
    vault.setMeta({
      path: "/tmp/test.kdbx",
      name: "Test",
      itemCount: 0,
      syncProvider: "local",
      entries: [],
      folders: [],
    });
    vault.setEntries([]);
    mocks.emptyRecycleBin.mockResolvedValue({ deletedEntries: 2 });
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

  it("provides navigation for each settings category", async () => {
    render(SettingsView);

    expect(
      await screen.findByRole("navigation", { name: "Settings sections" }),
    ).toBeInTheDocument();
    expect(screen.getByRole("link", { name: "Vault" })).toHaveAttribute("href", "#settings-vault");
    expect(screen.getByRole("link", { name: "Features" })).toHaveAttribute(
      "href",
      "#settings-features",
    );
    expect(screen.getByRole("link", { name: "Security" })).toHaveAttribute(
      "href",
      "#settings-security",
    );
    expect(screen.getByRole("link", { name: "About" })).toHaveAttribute("href", "#settings-about");
  });

  it("enables optional folder support", async () => {
    render(SettingsView);

    const toggle = await screen.findByRole("switch", { name: "Enable folders" });
    expect(toggle).not.toBeChecked();
    await fireEvent.click(toggle);

    expect(mocks.setFoldersEnabled).toHaveBeenCalledWith(true);
  });

  it("requests the shared empty-bin confirmation", async () => {
    vault.setEntries(
      ["trashed-1", "trashed-2"].map((id) => ({
        id,
        type: "login" as const,
        title: "Deleted",
        subtitle: "",
        tags: [],
        favorite: false,
        trashed: true,
      })),
    );
    render(SettingsView);

    await fireEvent.click(await screen.findByRole("button", { name: "Empty Recycle Bin…" }));

    expect(recycleBin.pending).toBe(true);
  });

  it("shows skipped entry names in a simple list", async () => {
    render(SettingsView);

    await fireEvent.click(screen.getByRole("button", { name: /Import 1Password 7/ }));
    await fireEvent.click(await screen.findByRole("button", { name: "View 2 skipped entries" }));

    expect(mocks.toastSuccess).toHaveBeenCalledWith("Imported 1 item (2 skipped).");
    expect(screen.getByRole("dialog", { name: "Entries not imported" })).toBeInTheDocument();
    const entries = screen.getAllByRole("listitem");
    expect(entries[0]).toHaveTextContent("Archived login");
    expect(entries[0]).toHaveTextContent("Item is in the 1Password trash");
    expect(entries[1]).toHaveTextContent("Unsupported document");
    expect(entries[1]).toHaveTextContent("The item couldn't be converted");
  });
});
