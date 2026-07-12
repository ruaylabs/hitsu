import * as entriesBridge from "$lib/bridge/entries";
import { selection } from "$lib/stores/selection.svelte";
import { toast } from "$lib/stores/toast.svelte";
import { vault } from "$lib/stores/vault.svelte";

interface PendingDelete {
  id: string;
  title: string;
  permanent: boolean;
  /** Extra cleanup run after a successful delete (e.g. exit edit mode). */
  onDeleted?: () => void;
}

let pending = $state<PendingDelete | null>(null);

/** Shared delete-with-confirmation flow. `request()` opens the single
 *  ConfirmDialog rendered in MainApp; `confirm()` deletes the entry and
 *  cleans up list + selection state in one place, so the ⌘⌫ shortcut and
 *  the detail-pane Delete button can't drift apart. */
export const entryDeletion = {
  get pending() {
    return pending;
  },
  request(id: string, title?: string, onDeleted?: () => void) {
    const item = vault.entries.find((e) => e.id === id);
    const resolved = title ?? item?.title ?? "this entry";
    pending = { id, title: resolved, permanent: item?.trashed ?? false, onDeleted };
  },
  cancel() {
    pending = null;
  },
  async confirm() {
    if (!pending) return;
    const { id, permanent, onDeleted } = pending;
    pending = null;
    try {
      if (permanent) await entriesBridge.entryDeletePermanent(id);
      else await entriesBridge.entryDelete(id);
    } catch (e) {
      console.error("Failed to delete entry", e);
      toast.error(e instanceof Error ? e.message : String(e));
      return;
    }
    // Caller cleanup first: it may clear state (newEntryId) that effects
    // keyed on the selection change below would otherwise act on.
    onDeleted?.();
    vault.setEntries(
      permanent
        ? vault.entries.filter((e) => e.id !== id)
        : vault.entries.map((e) => (e.id === id ? { ...e, trashed: true } : e)),
    );
    if (selection.selectedId === id) selection.selectedId = null;
  },
};
