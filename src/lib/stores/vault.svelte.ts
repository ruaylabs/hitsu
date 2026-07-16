import * as foldersBridge from "$lib/bridge/folders";
import * as prefsBridge from "$lib/bridge/prefs";
import type { EntrySummary, FolderSummary, VaultMeta } from "$lib/bridge/types";
import * as vaultBridge from "$lib/bridge/vault";
import { clipboard } from "$lib/stores/clipboard.svelte";
import { selection } from "$lib/stores/selection.svelte";

let vaultMeta = $state<VaultMeta | null>(null);
let entries = $state<EntrySummary[]>([]);
let folders = $state<FolderSummary[]>([]);
let locked = $state(false);
let editingId = $state<string | null>(null);
let creatingId = $state<string | null>(null);
let revision = $state(0);
let editSessionActive = $state(false);
let externalChangePending = $state(false);

function installOpenVault(meta: VaultMeta) {
  vaultMeta = meta;
  entries = meta.entries;
  folders = meta.folders ?? [];
  locked = false;
  revision += 1;
  externalChangePending = false;
  selection.selectedId = null;
  selection.search = "";
  selection.filter = { kind: "all" };
}

function installRefreshedVault(meta: VaultMeta) {
  vaultMeta = meta;
  entries = meta.entries;
  folders = meta.folders ?? [];
  revision += 1;
  externalChangePending = false;

  if (selection.selectedId && !entries.some((entry) => entry.id === selection.selectedId)) {
    selection.selectedId = null;
  }
  const filter = selection.filter;
  if (filter.kind === "folder" && !folders.some((folder) => folder.id === filter.folderId)) {
    selection.filter = { kind: "all" };
  }
}

function rememberVault(path: string) {
  void prefsBridge
    .prefsSetLastVault(path)
    .catch((error) => console.error("Failed to remember vault", error));
}

function normalizeError(error: unknown): Error {
  return error instanceof Error ? error : new Error(String(error));
}

function clearUnlockedState() {
  clipboard.cancel();
  selection.selectedId = null;
  selection.search = "";
  editingId = null;
  creatingId = null;
  editSessionActive = false;
  externalChangePending = false;
  locked = true;
  entries = [];
  folders = [];
}

export const vault = {
  get entries() {
    return entries;
  },
  get meta() {
    return vaultMeta;
  },
  get folders() {
    return folders;
  },
  get locked() {
    return locked;
  },
  get editingId() {
    return editingId;
  },
  setEditingId(id: string | null) {
    editingId = id;
  },
  get creatingId() {
    return creatingId;
  },
  setCreatingId(id: string | null) {
    creatingId = id;
  },
  get revision() {
    return revision;
  },
  get editSessionActive() {
    return editSessionActive;
  },
  setEditSessionActive(active: boolean) {
    editSessionActive = active;
  },
  get externalChangePending() {
    return externalChangePending;
  },
  setMeta(m: VaultMeta | null) {
    vaultMeta = m;
    if (!m) externalChangePending = false;
  },
  /** Open and install a vault, then remember it for startup and recent-vault UI. */
  async open(path: string, password: string) {
    try {
      const meta = await vaultBridge.vaultOpen(path, password);
      installOpenVault(meta);
      rememberVault(path);
      return meta;
    } catch (error) {
      throw normalizeError(error);
    }
  },
  /** Create and install a vault, then remember it for startup and recent-vault UI. */
  async create(path: string, password: string, name = "") {
    try {
      const meta = await vaultBridge.vaultCreate(path, password, name);
      installOpenVault(meta);
      rememberVault(path);
      return meta;
    } catch (error) {
      throw normalizeError(error);
    }
  },
  async refreshIfChanged() {
    try {
      const result = await vaultBridge.vaultRefreshIfChanged(!editSessionActive);
      if (result.reloaded && result.vault) {
        installRefreshedVault(result.vault);
      } else {
        externalChangePending = result.changed;
      }
      return result;
    } catch (error) {
      throw normalizeError(error);
    }
  },
  setEntries(data: EntrySummary[]) {
    entries = data;
  },
  setFolders(data: FolderSummary[]) {
    folders = data;
  },
  async createFolder(parentId: string | null, name: string) {
    const folder = await foldersBridge.folderCreate(parentId, name);
    folders = [...folders, folder];
    return folder;
  },
  async renameFolder(id: string, name: string) {
    const updated = await foldersBridge.folderRename(id, name);
    folders = folders.map((folder) => (folder.id === id ? updated : folder));
    return updated;
  },
  folderIdsWithin(id: string) {
    const ids = new Set([id]);
    let changed = true;
    while (changed) {
      changed = false;
      for (const folder of folders) {
        if (folder.parentId && ids.has(folder.parentId) && !ids.has(folder.id)) {
          ids.add(folder.id);
          changed = true;
        }
      }
    }
    return ids;
  },
  async lock() {
    if (locked) return;
    // Drop decrypted vault from the Rust backend (zeroizes master key in memory)
    try {
      await vaultBridge.vaultLock();
    } catch (e) {
      console.error("Failed to lock vault in backend", e);
    }
    // Forget all decrypted-entry UI state even if the backend command failed.
    clearUnlockedState();
  },
  /** Apply an OS-initiated lock after the backend has already dropped its
   * decrypted database and cleared the system clipboard. */
  sessionLocked() {
    clearUnlockedState();
  },
  unlock() {
    locked = false;
  },
};
