import { fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { recycleBin } from "$lib/stores/recycleBin.svelte";
import { vault } from "$lib/stores/vault.svelte";
import SettingsView from "./SettingsView.svelte";

const mocks = vi.hoisted(() => ({
  import1pif: vi.fn(),
  exportImportReport: vi.fn(),
  saveDialog: vi.fn(),
  setFoldersEnabled: vi.fn(),
  emptyRecycleBin: vi.fn(),
  toastSuccess: vi.fn(),
  toastError: vi.fn(),
  setTheme: vi.fn(),
}));

vi.mock("@tauri-apps/plugin-dialog", () => ({
  open: vi.fn(),
  save: mocks.saveDialog,
}));

vi.mock("$lib/bridge/prefs", () => ({
  prefsGet: vi.fn().mockResolvedValue({
    idleLockMinutes: 5,
    clipboardClearSeconds: 15,
    foldersEnabled: false,
    theme: "system",
    recentVaults: [],
  }),
  prefsSetSecurity: vi.fn(),
  prefsSetTheme: mocks.setTheme,
  prefsSetFoldersEnabled: mocks.setFoldersEnabled,
}));

vi.mock("$lib/bridge/vault", () => ({
  vaultImport1pif: mocks.import1pif,
  importReportExport: mocks.exportImportReport,
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
    document.documentElement.removeAttribute("data-theme");
    mocks.setTheme.mockResolvedValue(undefined);
    mocks.saveDialog.mockResolvedValue(undefined);
    mocks.exportImportReport.mockResolvedValue(true);
    mocks.emptyRecycleBin.mockResolvedValue({ deletedEntries: 2 });
    mocks.import1pif.mockResolvedValue({
      importedItems: 1,
      importedAttachments: 0,
      skippedItems: 2,
      failedItems: 1,
      skippedEntries: [
        {
          title: "Archived login",
          reason: "Item is in the 1Password trash",
          failed: false,
        },
        {
          title: "Unsupported document",
          reason: "The item couldn't be converted",
          failed: true,
        },
      ],
      entries: [
        {
          id: "imported-1",
          type: "login",
          title: "Imported login",
          subtitle: "user@example.com",
          tags: [],
          favorite: false,
        },
      ],
    });
  });

  it("provides navigation for each settings category", async () => {
    render(SettingsView);

    expect(
      await screen.findByRole("navigation", { name: "Settings sections" }),
    ).toBeInTheDocument();
    expect(screen.getByRole("link", { name: "Vault" })).toHaveAttribute("href", "#settings-vault");
    expect(screen.getByRole("link", { name: "Appearance" })).toHaveAttribute(
      "href",
      "#settings-appearance",
    );
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

  it("applies and saves a theme override", async () => {
    render(SettingsView);

    const select = await screen.findByRole("combobox", { name: "Theme" });
    await waitFor(() => expect(select).toHaveValue("system"));
    await fireEvent.change(select, { target: { value: "dark" } });

    expect(document.documentElement).toHaveAttribute("data-theme", "dark");
    expect(mocks.setTheme).toHaveBeenCalledWith("dark");
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

  it("confirms the destination and shows progress while importing", async () => {
    let finishImport: (value: null) => void = () => {};
    mocks.import1pif.mockReturnValueOnce(
      new Promise((resolve) => {
        finishImport = resolve;
      }),
    );
    render(SettingsView);

    await fireEvent.click(screen.getByRole("button", { name: /Import 1Password 7/ }));

    expect(screen.getByRole("dialog", { name: "Import 1Password 7 data" })).toHaveTextContent(
      "Import into Test?",
    );
    expect(screen.getByText(/will be merged into this vault/)).toBeInTheDocument();
    expect(mocks.import1pif).not.toHaveBeenCalled();

    await fireEvent.click(screen.getByRole("button", { name: "Choose export…" }));
    expect(screen.getByRole("status", { name: "Import in progress" })).toBeInTheDocument();

    finishImport(null);
    await waitFor(() =>
      expect(screen.getByRole("button", { name: /Import 1Password 7/ })).toBeEnabled(),
    );
  });

  it("keeps a persistent completion summary", async () => {
    render(SettingsView);

    await fireEvent.click(screen.getByRole("button", { name: /Import 1Password 7/ }));
    await fireEvent.click(screen.getByRole("button", { name: "Choose export…" }));

    const summary = await screen.findByText("Import complete");
    expect(summary.closest(".import-summary")).toHaveTextContent(
      /Imported\s*1\s*Attachments\s*0\s*Skipped\s*1\s*Failed\s*1/,
    );
    expect(mocks.toastSuccess).toHaveBeenCalledWith("Imported 1 item. 1 skipped. 1 failed.");
    expect(screen.queryByText("Imported login")).not.toBeInTheDocument();
  });

  it("shows not-imported details and exports them as CSV", async () => {
    render(SettingsView);

    await fireEvent.click(screen.getByRole("button", { name: /Import 1Password 7/ }));
    await fireEvent.click(screen.getByRole("button", { name: "Choose export…" }));
    await fireEvent.click(
      await screen.findByRole("button", { name: "Review 2 items not imported" }),
    );

    expect(screen.getByRole("dialog", { name: "Items not imported" })).toBeInTheDocument();
    const entries = screen.getAllByRole("listitem");
    expect(entries[0]).toHaveTextContent("Skipped Archived login");
    expect(entries[0]).toHaveTextContent("Item is in the 1Password trash");
    expect(entries[1]).toHaveTextContent("Failed Unsupported document");
    expect(entries[1]).toHaveTextContent("The item couldn't be converted");

    await fireEvent.click(screen.getByRole("button", { name: "Export CSV…" }));

    expect(mocks.exportImportReport).toHaveBeenCalledWith(
      expect.stringContaining('"Failed","Unsupported document","The item couldn\'t be converted"'),
    );
  });
});
