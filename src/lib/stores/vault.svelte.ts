import type { Entry } from "$lib/bridge/types";

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
  lock() {
    locked = true;
    entries = [];
  },
  unlock() {
    locked = false;
  },
};
