import type { EntrySummary, VaultMeta } from "$lib/bridge/types";
import * as vaultBridge from "$lib/bridge/vault";
import { clipboard } from "$lib/stores/clipboard.svelte";
import { selection } from "$lib/stores/selection.svelte";

let vaultMeta = $state<VaultMeta | null>(null);
let entries = $state<EntrySummary[]>([]);
let locked = $state(false);
let editingId = $state<string | null>(null);
let creatingId = $state<string | null>(null);

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
  /** Install a freshly opened/created vault. Resets the per-vault UI state
   *  (selected entry, search, sidebar filter) so nothing from a previously
   *  open vault leaks into this one — a stale selectedId would make
   *  ItemDetail fetch an entry the new vault doesn't have. */
  openVault(meta: VaultMeta) {
    vaultMeta = meta;
    entries = meta.entries;
    locked = false;
    selection.selectedId = null;
    selection.search = "";
    selection.filter = { kind: "all" };
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
    // Clear any pending clipboard auto-clear timer
    clipboard.cancel();
    // Forget the selected entry and search query so nothing sensitive
    // auto-loads or re-renders after unlock.
    selection.selectedId = null;
    selection.search = "";
    locked = true;
    entries = [];
  },
  unlock() {
    locked = false;
  },
};
