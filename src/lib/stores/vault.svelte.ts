import * as prefsBridge from "$lib/bridge/prefs";
import type { EntrySummary, VaultMeta } from "$lib/bridge/types";
import * as vaultBridge from "$lib/bridge/vault";
import { clipboard } from "$lib/stores/clipboard.svelte";
import { selection } from "$lib/stores/selection.svelte";

let vaultMeta = $state<VaultMeta | null>(null);
let entries = $state<EntrySummary[]>([]);
let locked = $state(false);
let editingId = $state<string | null>(null);
let creatingId = $state<string | null>(null);

function installOpenVault(meta: VaultMeta) {
  vaultMeta = meta;
  entries = meta.entries;
  locked = false;
  selection.selectedId = null;
  selection.search = "";
  selection.filter = { kind: "all" };
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
  locked = true;
  entries = [];
}

export const vault = {
  get entries() {
    return entries;
  },
  get meta() {
    return vaultMeta;
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
  setMeta(m: VaultMeta | null) {
    vaultMeta = m;
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
  setEntries(data: EntrySummary[]) {
    entries = data;
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
