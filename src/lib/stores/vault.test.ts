import { beforeEach, describe, expect, it, vi } from "vitest";
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
  vault.setCreatingId(null);
  vault.setEditingId(null);
});

describe("vault store", () => {
  it("opens, installs, and remembers a vault", async () => {
    selection.selectedId = "stale-entry";
    selection.search = "stale search";
    selection.filter = { kind: "favorites" };
    const meta: VaultMeta = {
      path: "/tmp/test.kdbx",
      name: "Test vault",
      itemCount: 1,
      syncProvider: "local",
      entries: [firstEntry],
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

  it("creates, installs, and remembers a vault", async () => {
    const meta: VaultMeta = {
      path: "/tmp/new.kdbx",
      name: "New vault",
      itemCount: 0,
      syncProvider: "local",
      entries: [],
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
