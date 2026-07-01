import type { EntrySummary, VaultMeta } from "$lib/bridge/types";
import * as vaultBridge from "$lib/bridge/vault";
import { clipboard } from "$lib/stores/clipboard.svelte";

let vaultMeta = $state<VaultMeta | null>(null);
let entries = $state<EntrySummary[]>([]);
let locked = $state(false);
let editingId = $state<string | null>(null);

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
  setMeta(m: VaultMeta | null) {
    vaultMeta = m;
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
    locked = true;
    entries = [];
  },
  unlock() {
    locked = false;
  },
};
