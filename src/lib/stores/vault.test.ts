import { beforeEach, describe, expect, it, vi } from "vitest";
import * as biometricBridge from "$lib/bridge/biometric";
import * as entriesBridge from "$lib/bridge/entries";
import * as foldersBridge from "$lib/bridge/folders";
import * as prefsBridge from "$lib/bridge/prefs";
import type { EntrySummary, VaultMeta } from "$lib/bridge/types";
import * as vaultBridge from "$lib/bridge/vault";
import { clipboard } from "./clipboard.svelte";
import { selection } from "./selection.svelte";
import { vault } from "./vault.svelte";

const firstEntry: EntrySummary = {
  id: "first",
  type: "password",
  title: "First",
  subtitle: "",
  tags: [],
  favorite: false,
};

const secondEntry: EntrySummary = {
  id: "second",
  type: "login",
  title: "Second",
  subtitle: "alice",
  tags: [],
  favorite: false,
};

beforeEach(() => {
  vi.restoreAllMocks();
  vi.spyOn(prefsBridge, "prefsSetLastVault").mockResolvedValue(undefined);
  selection.selectedId = null;
  selection.search = "";
  selection.filter = { kind: "all" };
  vault.unlock();
  vault.setMeta(null);
  vault.setEntries([]);
  vault.setFolders([]);
  vault.setCreatingId(null);
  vault.setEditingId(null);
  vault.setEditSessionActive(false);
});

describe("vault store", () => {
  it("opens, installs, and remembers a vault", async () => {
    selection.selectedId = "stale-entry";
    selection.search = "stale search";
    selection.filter = { kind: "favorites" };
    const meta: VaultMeta = {
      path: "/tmp/test.kdbx",
      name: "Test vault",
      entries: [firstEntry],
      folders: [],
    };
    const open = vi.spyOn(vaultBridge, "vaultOpen").mockResolvedValue(meta);

    await vault.open(meta.path, "master-password");

    expect(open).toHaveBeenCalledWith(meta.path, "master-password");
    expect(prefsBridge.prefsSetLastVault).toHaveBeenCalledWith(meta.path);
    expect(vault.meta).toEqual(meta);
    expect(vault.entries).toEqual([firstEntry]);
    expect(selection.selectedId).toBeNull();
    expect(selection.search).toBe("");
    expect(selection.filter).toEqual({ kind: "all" });

    vault.setEntries([secondEntry]);
    expect(vault.entries).toEqual([secondEntry]);
  });

  it("unlocks with Touch ID, installs, and remembers a vault", async () => {
    const meta: VaultMeta = {
      path: "/tmp/biometric.kdbx",
      name: "Biometric vault",
      entries: [firstEntry],
      folders: [],
    };
    const unlock = vi.spyOn(biometricBridge, "biometricUnlock").mockResolvedValue(meta);

    await vault.unlockWithBiometric(meta.path);

    expect(unlock).toHaveBeenCalledWith(meta.path);
    expect(prefsBridge.prefsSetLastVault).toHaveBeenCalledWith(meta.path);
    expect(vault.meta).toEqual(meta);
    expect(vault.entries).toEqual([firstEntry]);
    expect(vault.locked).toBe(false);
  });

  it("creates, installs, and remembers a vault", async () => {
    const meta: VaultMeta = {
      path: "/tmp/new.kdbx",
      name: "New vault",
      entries: [],
      folders: [],
    };
    const create = vi.spyOn(vaultBridge, "vaultCreate").mockResolvedValue(meta);

    await vault.create(meta.path, "master-password", "New vault");

    expect(create).toHaveBeenCalledWith(meta.path, "master-password", "New vault");
    expect(prefsBridge.prefsSetLastVault).toHaveBeenCalledWith(meta.path);
    expect(vault.meta).toEqual(meta);
    expect(vault.locked).toBe(false);
  });

  it("normalizes backend failures to errors without replacing state", async () => {
    vi.spyOn(vaultBridge, "vaultOpen").mockRejectedValue("Wrong password");

    await expect(vault.open("/tmp/test.kdbx", "wrong")).rejects.toThrow("Wrong password");

    expect(vault.meta).toBeNull();
    expect(prefsBridge.prefsSetLastVault).not.toHaveBeenCalled();
  });

  it("normalizes Touch ID failures without replacing state", async () => {
    vi.spyOn(biometricBridge, "biometricUnlock").mockRejectedValue({
      kind: "biometric_failed",
      message: "Touch ID failed",
    });

    await expect(vault.unlockWithBiometric("/tmp/test.kdbx")).rejects.toMatchObject({
      kind: "biometric_failed",
      message: "Touch ID failed",
    });

    expect(vault.meta).toBeNull();
    expect(prefsBridge.prefsSetLastVault).not.toHaveBeenCalled();
  });

  it("creates and renames folders in local state", async () => {
    const parent = { id: "work", name: "Work" };
    const child = { id: "clients", name: "Clients", parentId: "work" };
    vi.spyOn(foldersBridge, "folderCreate").mockResolvedValue(child);
    vi.spyOn(foldersBridge, "folderRename").mockResolvedValue({ ...child, name: "Customers" });
    vault.setFolders([parent]);

    await vault.createFolder(parent.id, child.name);
    await vault.renameFolder(child.id, "Customers");

    expect(foldersBridge.folderCreate).toHaveBeenCalledWith("work", "Clients");
    expect(foldersBridge.folderRename).toHaveBeenCalledWith("clients", "Customers");
    expect(vault.folders).toEqual([parent, { ...child, name: "Customers" }]);
  });

  it("updates tag summaries and invalidates selected entry details", async () => {
    const tagRename = vi.spyOn(entriesBridge, "tagRename").mockResolvedValue(undefined);
    const tagDelete = vi.spyOn(entriesBridge, "tagDelete").mockResolvedValue(undefined);
    const refresh = vi.spyOn(vaultBridge, "vaultRefreshIfChanged");
    vault.setEntries([{ ...firstEntry, tags: ["work", "shared"] }]);
    const initialRevision = vault.revision;

    await vault.renameTag("work", "office");

    expect(tagRename).toHaveBeenCalledWith("work", "office");
    expect(vault.entries[0].tags).toEqual(["office", "shared"]);
    expect(vault.revision).toBe(initialRevision + 1);

    await vault.deleteTag("shared");

    expect(tagDelete).toHaveBeenCalledWith("shared");
    expect(vault.entries[0].tags).toEqual(["office"]);
    expect(vault.revision).toBe(initialRevision + 2);
    expect(refresh).not.toHaveBeenCalled();
  });

  it("installs external changes while preserving the current view", async () => {
    const refreshedEntry = { ...secondEntry, title: "Changed in KeePassXC" };
    const refreshedMeta: VaultMeta = {
      path: "/tmp/test.kdbx",
      name: "Test vault",
      entries: [refreshedEntry],
      folders: [],
    };
    const refresh = vi.spyOn(vaultBridge, "vaultRefreshIfChanged").mockResolvedValue({
      changed: true,
      reloaded: true,
      vault: refreshedMeta,
    });
    vault.setMeta({ ...refreshedMeta, entries: [secondEntry] });
    vault.setEntries([secondEntry]);
    selection.selectedId = secondEntry.id;
    selection.search = "second";
    selection.filter = { kind: "favorites" };
    const revision = vault.revision;

    const result = await vault.refreshIfChanged();

    expect(refresh).toHaveBeenCalledWith(true);
    expect(result.reloaded).toBe(true);
    expect(vault.entries).toEqual([refreshedEntry]);
    expect(vault.revision).toBe(revision + 1);
    expect(selection.selectedId).toBe(secondEntry.id);
    expect(selection.search).toBe("second");
    expect(selection.filter).toEqual({ kind: "favorites" });
  });

  it("defers an external reload while an edit session is active", async () => {
    const refresh = vi.spyOn(vaultBridge, "vaultRefreshIfChanged").mockResolvedValue({
      changed: true,
      reloaded: false,
      vault: null,
    });
    vault.setEditSessionActive(true);

    await vault.refreshIfChanged();

    expect(refresh).toHaveBeenCalledWith(false);
    expect(vault.externalChangePending).toBe(true);
  });

  it("locks frontend state when the backend lock rejects", async () => {
    vault.setEntries([firstEntry]);
    vault.setEditingId(firstEntry.id);
    selection.selectedId = firstEntry.id;
    const lock = vi.spyOn(vaultBridge, "vaultLock").mockRejectedValue(new Error("IPC failed"));
    const cancelClipboard = vi.spyOn(clipboard, "cancel").mockImplementation(() => {});
    const consoleError = vi.spyOn(console, "error").mockImplementation(() => {});

    await vault.lock();

    expect(lock).toHaveBeenCalledOnce();
    expect(vault.locked).toBe(true);
    expect(vault.entries).toEqual([]);
    expect(vault.editingId).toBeNull();
    expect(selection.selectedId).toBeNull();
    expect(consoleError).toHaveBeenCalledWith("Failed to lock vault in backend", expect.any(Error));

    lock.mockRestore();
    cancelClipboard.mockRestore();
    consoleError.mockRestore();
  });

  it("switches to another vault by locking and prompting for the new path", async () => {
    const meta: VaultMeta = {
      path: "/tmp/test.kdbx",
      name: "Test vault",
      entries: [firstEntry],
      folders: [],
    };
    vault.setMeta(meta);
    vault.setEntries([firstEntry]);
    selection.selectedId = firstEntry.id;
    const lock = vi.spyOn(vaultBridge, "vaultLock").mockResolvedValue(undefined);

    await vault.switchTo("/tmp/other.kdbx");

    expect(lock).toHaveBeenCalledOnce();
    expect(vault.locked).toBe(true);
    expect(vault.entries).toEqual([]);
    expect(selection.selectedId).toBeNull();
    expect(vault.meta).toMatchObject({ path: "/tmp/other.kdbx", name: "other.kdbx" });
  });

  it("applies an OS session lock without another backend call", () => {
    const lock = vi.spyOn(vaultBridge, "vaultLock");
    vault.setEntries([firstEntry]);
    vault.setCreatingId(firstEntry.id);
    selection.selectedId = firstEntry.id;

    vault.sessionLocked();

    expect(lock).not.toHaveBeenCalled();
    expect(vault.locked).toBe(true);
    expect(vault.entries).toEqual([]);
    expect(vault.creatingId).toBeNull();
    expect(selection.selectedId).toBeNull();
    lock.mockRestore();
  });
});
