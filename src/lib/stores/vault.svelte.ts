import type { Entry } from "$lib/bridge/types";

let vaultMeta = $state<{ path: string; name: string; itemCount: number } | null>(null);
let entries = $state<Entry[]>([]);
let locked = $state(false);

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
