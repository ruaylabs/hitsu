import { onDestroy } from "svelte";
import { toast } from "$lib/stores/toast.svelte";
import { errorMessage } from "$lib/utils/errorMessage";

/** Shared copy feedback, tier 1: toast-only. Used by copy affordances
 *  without an inline icon (context menu items, keyboard shortcuts).
 *  Returns true when the copy succeeded. */
export async function runCopyFeedback(
  copy: () => void | Promise<void>,
  successMessage: string,
): Promise<boolean> {
  try {
    await copy();
  } catch (error) {
    toast.error(errorMessage(error));
    return false;
  }
  toast.success(successMessage);
  return true;
}

/** Shared copy feedback, tier 2: toast + inline icon→check swap. Used where
 *  the copy action has its own button in the UI (detail fields, generator). */
export function createCopyFeedback(durationMs = 1000) {
  let active = $state(false);
  let timer: ReturnType<typeof setTimeout> | undefined;
  function show() {
    clearTimeout(timer);
    active = true;
    timer = setTimeout(() => (active = false), durationMs);
  }
  onDestroy(() => clearTimeout(timer));
  return {
    get active() {
      return active;
    },
    show,
    async run(copy: () => void | Promise<void>, successMessage: string) {
      if (await runCopyFeedback(copy, successMessage)) show();
    },
  };
}
