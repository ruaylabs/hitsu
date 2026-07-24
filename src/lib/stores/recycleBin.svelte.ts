import * as vaultBridge from "$lib/bridge/vault";
import { selection } from "$lib/stores/selection.svelte";
import { toast } from "$lib/stores/toast.svelte";
import { vault } from "$lib/stores/vault.svelte";

let pending = $state(false);
let emptying = $state(false);

export const recycleBin = {
  get pending() {
    return pending;
  },
  get emptying() {
    return emptying;
  },
  get count() {
    return vault.entries.filter((entry) => entry.trashed).length;
  },
  requestEmpty() {
    if (!vault.entries.some((entry) => entry.trashed) || emptying) return;
    pending = true;
  },
  cancel() {
    pending = false;
  },
  async confirm() {
    if (!pending || emptying) return;
    pending = false;
    emptying = true;
    try {
      const result = await vaultBridge.vaultEmptyRecycleBin();
      const selectedWasTrashed = vault.entries.some(
        (entry) => entry.id === selection.selectedId && entry.trashed,
      );
      const entries = vault.entries.filter((entry) => !entry.trashed);
      vault.setEntries(entries);
      if (vault.meta) {
        vault.setMeta({ ...vault.meta, entries, itemCount: entries.length });
      }
      if (selectedWasTrashed) selection.selectedId = null;
      toast.success(
        result.deletedEntries === 0
          ? "Recycle Bin is already empty"
          : `Permanently deleted ${result.deletedEntries} entr${result.deletedEntries === 1 ? "y" : "ies"}`,
      );
    } catch (error) {
      toast.error(error instanceof Error ? error.message : String(error));
    } finally {
      emptying = false;
    }
  },
};
