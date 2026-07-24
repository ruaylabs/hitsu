import * as entriesBridge from "$lib/bridge/entries";
import { selection } from "$lib/stores/selection.svelte";
import { toast } from "$lib/stores/toast.svelte";
import { vault } from "$lib/stores/vault.svelte";
import { errorMessage } from "$lib/utils/errorMessage";

interface PendingDelete {
  id: string;
  title: string;
  permanent: true;
  /** Extra cleanup run after a successful delete (e.g. exit edit mode). */
  onDeleted?: () => void;
}

let pending = $state<PendingDelete | null>(null);
const deletingIds = new Set<string>();

async function restoreEntry(id: string, title: string) {
  try {
    await entriesBridge.entryRestore(id);
    vault.setEntries(
      vault.entries.map((entry) => (entry.id === id ? { ...entry, trashed: false } : entry)),
    );
    toast.success(`Restored "${title}"`);
  } catch (error) {
    console.error("Failed to restore entry", error);
    toast.error(errorMessage(error));
  }
}

async function moveToRecycleBin(id: string, title: string, onDeleted?: () => void) {
  if (deletingIds.has(id)) return;
  deletingIds.add(id);
  try {
    await entriesBridge.entryDelete(id);
  } catch (error) {
    console.error("Failed to delete entry", error);
    toast.error(errorMessage(error));
    return;
  } finally {
    deletingIds.delete(id);
  }

  onDeleted?.();
  vault.setEntries(
    vault.entries.map((entry) => (entry.id === id ? { ...entry, trashed: true } : entry)),
  );
  if (selection.selectedId === id) selection.selectedId = null;
  toast.info(`Moved "${title}" to Recycle Bin`, 8000, {
    label: "Undo",
    run: () => restoreEntry(id, title),
  });
}

/** Active entries move directly to the Recycle Bin with an undo action.
 * Entries already in the bin require confirmation before permanent deletion. */
export const entryDeletion = {
  get pending() {
    return pending;
  },
  async request(id: string, title?: string, onDeleted?: () => void) {
    const item = vault.entries.find((entry) => entry.id === id);
    const resolved = title ?? item?.title ?? "this entry";
    if (!item?.trashed) {
      await moveToRecycleBin(id, resolved, onDeleted);
      return;
    }
    pending = { id, title: resolved, permanent: true, onDeleted };
  },
  cancel() {
    pending = null;
  },
  async confirm() {
    if (!pending) return;
    const { id, onDeleted } = pending;
    pending = null;
    try {
      await entriesBridge.entryDeletePermanent(id);
    } catch (error) {
      console.error("Failed to delete entry", error);
      toast.error(errorMessage(error));
      return;
    }
    onDeleted?.();
    vault.setEntries(vault.entries.filter((entry) => entry.id !== id));
    if (selection.selectedId === id) selection.selectedId = null;
  },
};
