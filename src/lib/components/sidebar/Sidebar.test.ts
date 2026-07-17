import { fireEvent, render, screen } from "@testing-library/svelte";
import { beforeEach, describe, expect, it, vi } from "vitest";
import * as foldersBridge from "$lib/bridge/folders";
import type { EntrySummary } from "$lib/bridge/types";
import { features } from "$lib/stores/features.svelte";
import { selection } from "$lib/stores/selection.svelte";
import { vault } from "$lib/stores/vault.svelte";
import Sidebar from "./Sidebar.svelte";

const entries: EntrySummary[] = [
  { id: "1", type: "login", title: "Login", subtitle: "", tags: [], favorite: false },
  { id: "2", type: "password", title: "Password", subtitle: "", tags: [], favorite: false },
  { id: "3", type: "password", title: "Password 2", subtitle: "", tags: [], favorite: false },
];

beforeEach(() => {
  vi.restoreAllMocks();
  localStorage.clear();
  selection.selectedId = null;
  selection.search = "";
  selection.filter = { kind: "all" };
  vault.setEntries(entries);
  vault.setFolders([]);
  features.hydrate({
    lastVault: null,
    recentVaults: [],
    idleLockMinutes: 5,
    clipboardClearSeconds: 15,
    foldersEnabled: false,
    kdfUpgradeDismissedVaults: [],
  });
});

describe("Sidebar", () => {
  it("shows type counts and selects the password filter", async () => {
    const passwords = render(Sidebar).getByRole("tab", { name: "Passwords 2" });

    expect(passwords).toHaveAttribute("aria-selected", "false");
    await fireEvent.click(passwords);

    expect(selection.filter).toEqual({ kind: "type", type: "password" });
    expect(screen.getByRole("tab", { name: "Passwords 2" })).toHaveAttribute(
      "aria-selected",
      "true",
    );
  });

  it("counts trashed entries separately and opens the recycle bin", async () => {
    vault.setEntries([...entries, { ...entries[0], id: "deleted", trashed: true }]);
    const recycleBin = render(Sidebar).getByRole("tab", { name: "Recycle Bin 1" });

    expect(screen.getByRole("tab", { name: "All items 3" })).toBeInTheDocument();
    await fireEvent.click(recycleBin);

    expect(selection.filter).toEqual({ kind: "trash" });
  });

  it("shows nested folders only when the feature is enabled", async () => {
    vault.setFolders([
      { id: "work", name: "Work" },
      { id: "clients", name: "Clients", parentId: "work" },
    ]);
    vault.setEntries([
      { ...entries[0], folderId: "work" },
      { ...entries[1], folderId: "clients" },
    ]);

    const disabled = render(Sidebar);
    expect(screen.queryByRole("tab", { name: "Work 2" })).not.toBeInTheDocument();
    disabled.unmount();

    features.hydrate({
      lastVault: null,
      recentVaults: [],
      idleLockMinutes: 5,
      clipboardClearSeconds: 15,
      foldersEnabled: true,
      kdfUpgradeDismissedVaults: [],
    });
    render(Sidebar);

    const work = screen.getByRole("tab", { name: "Work 2" });
    expect(screen.getByRole("tab", { name: "Clients 1" })).toBeInTheDocument();
    await fireEvent.click(work);
    expect(selection.filter).toEqual({ kind: "folder", folderId: "work" });
  });

  it("creates nested folders and renames them", async () => {
    features.hydrate({
      lastVault: null,
      recentVaults: [],
      idleLockMinutes: 5,
      clipboardClearSeconds: 15,
      foldersEnabled: true,
      kdfUpgradeDismissedVaults: [],
    });
    vault.setFolders([{ id: "work", name: "Work" }]);
    vi.spyOn(foldersBridge, "folderCreate").mockResolvedValue({
      id: "clients",
      name: "Clients",
      parentId: "work",
    });
    vi.spyOn(foldersBridge, "folderRename").mockResolvedValue({
      id: "clients",
      name: "Customers",
      parentId: "work",
    });
    render(Sidebar);

    await fireEvent.click(screen.getByRole("button", { name: "Add folder inside Work" }));
    await fireEvent.input(screen.getByLabelText("Name"), { target: { value: "Clients" } });
    await fireEvent.click(screen.getByRole("button", { name: "Create" }));

    expect(foldersBridge.folderCreate).toHaveBeenCalledWith("work", "Clients");
    await fireEvent.click(await screen.findByRole("button", { name: "Rename Clients" }));
    await fireEvent.input(screen.getByLabelText("Name"), { target: { value: "Customers" } });
    await fireEvent.click(screen.getByRole("button", { name: "Rename" }));

    expect(foldersBridge.folderRename).toHaveBeenCalledWith("clients", "Customers");
    expect(await screen.findByRole("tab", { name: "Customers 0" })).toBeInTheDocument();
  });

  it("persists the collapsed tags state", async () => {
    vault.setEntries([{ ...entries[0], tags: ["work"] }]);
    const sidebar = render(Sidebar);

    const collapseButton = screen.getByRole("button", { name: "Collapse Tags" });
    expect(screen.getByRole("tab", { name: "work" })).toBeInTheDocument();
    expect(collapseButton).toHaveAttribute("aria-expanded", "true");

    await fireEvent.click(collapseButton);

    expect(screen.queryByRole("tab", { name: "work" })).not.toBeInTheDocument();
    expect(localStorage.getItem("hitsu:sidebar-tags-collapsed")).toBe("true");
    sidebar.unmount();

    render(Sidebar);

    const expandButton = await screen.findByRole("button", { name: "Expand Tags" });
    expect(expandButton).toHaveAttribute("aria-expanded", "false");
    expect(screen.queryByRole("tab", { name: "work" })).not.toBeInTheDocument();

    await fireEvent.click(expandButton);

    expect(screen.getByRole("tab", { name: "work" })).toBeInTheDocument();
    expect(localStorage.getItem("hitsu:sidebar-tags-collapsed")).toBe("false");
  });
});
