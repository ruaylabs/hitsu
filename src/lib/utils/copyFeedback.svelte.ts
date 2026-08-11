import { onDestroy } from "svelte";
import { toast } from "$lib/stores/toast.svelte";
import { errorMessage } from "$lib/utils/errorMessage";
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
      try {
        await copy();
      } catch (error) {
        toast.error(errorMessage(error));
        return;
      }
      toast.success(successMessage);
      show();
    },
  };
}
