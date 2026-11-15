<script lang="ts">
  import "@tabler/icons-webfont/dist/tabler-icons.css";
  import "../lib/styles/tokens.css";
  import "../lib/styles/reset.css";
  import "../lib/styles/global.css";
  import "../lib/styles/controls.css";
  import { onMount } from "svelte";
  import ToastStack from "$lib/components/ui/ToastStack.svelte";
  import { toast } from "$lib/stores/toast.svelte";
  import { errorMessage, isIpcErrorPayload } from "$lib/utils/errorMessage";
  import { isMacPlatform } from "$lib/utils/keyboardShortcut";

  function rejectionMessage(reason: unknown) {
    if (reason instanceof Error || typeof reason === "string" || isIpcErrorPayload(reason)) {
      const message = errorMessage(reason).trim();
      if (message) return message;
    }
    return "An unexpected background error occurred";
  }

  onMount(() => {
    document.documentElement.dataset.platform = isMacPlatform()
      ? "macos"
      : navigator.platform.toLowerCase().includes("linux")
        ? "linux"
        : "other";

    const handleUnhandledRejection = (event: PromiseRejectionEvent) => {
      // We report the rejection ourselves so it is visible to the user while
      // retaining the diagnostic in the developer console.
      event.preventDefault();
      console.error("Unhandled promise rejection", event.reason);
      toast.error(rejectionMessage(event.reason));
    };

    const handleContextMenu = (event: MouseEvent) => {
      if (import.meta.env.DEV || event.defaultPrevented || !(event.target instanceof Element))
        return;
      if (
        event.target.closest("input, textarea") ||
        (event.target as HTMLElement).isContentEditable
      ) {
        return;
      }
      for (let element: Element | null = event.target; element; element = element.parentElement) {
        const style = getComputedStyle(element);
        const selection = style.userSelect || style.getPropertyValue("-webkit-user-select");
        if (selection === "text" || selection === "all") return;
        if (selection === "none") break;
      }
      event.preventDefault();
    };

    window.addEventListener("unhandledrejection", handleUnhandledRejection);
    window.addEventListener("contextmenu", handleContextMenu);
    return () => {
      window.removeEventListener("unhandledrejection", handleUnhandledRejection);
      window.removeEventListener("contextmenu", handleContextMenu);
    };
  });
</script>

<slot />
<ToastStack />
