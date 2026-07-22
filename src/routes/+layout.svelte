<script lang="ts">
  import "@tabler/icons-webfont/dist/tabler-icons.css";
  import "../lib/styles/tokens.css";
  import "../lib/styles/reset.css";
  import "../lib/styles/global.css";
  import "../lib/styles/controls.css";
  import { onMount } from "svelte";
  import ToastStack from "$lib/components/ui/ToastStack.svelte";
  import { toast } from "$lib/stores/toast.svelte";

  function rejectionMessage(reason: unknown) {
    if (reason instanceof Error && reason.message.trim()) return reason.message;
    if (typeof reason === "string" && reason.trim()) return reason;
    return "An unexpected background error occurred";
  }

  onMount(() => {
    const handleUnhandledRejection = (event: PromiseRejectionEvent) => {
      // We report the rejection ourselves so it is visible to the user while
      // retaining the diagnostic in the developer console.
      event.preventDefault();
      console.error("Unhandled promise rejection", event.reason);
      toast.error(rejectionMessage(event.reason));
    };

    window.addEventListener("unhandledrejection", handleUnhandledRejection);
    return () => window.removeEventListener("unhandledrejection", handleUnhandledRejection);
  });
</script>

<slot />
<ToastStack />
