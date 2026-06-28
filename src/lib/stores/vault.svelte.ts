import type { Entry } from "$lib/bridge/types";
import * as vaultBridge from "$lib/bridge/vault";
import { clipboard } from "$lib/stores/clipboard.svelte";

let vaultMeta = $state<{ path: string; name: string; itemCount: number } | null>(null);
let entries = $state<Entry[]>([]);
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
  setMeta(m: { path: string; name: string; itemCount: number } | null) {
    vaultMeta = m;
  },
  getEntry(id: string): Entry | undefined {
    return entries.find((e) => e.id === id);
  },
  setEntries(data: Entry[]) {
    entries = data;
  },
  async lock() {
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
